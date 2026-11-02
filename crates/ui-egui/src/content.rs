//! The Content panel: thumbnails, details and list views, and the filmstrip row (docs/ui-design.md §4.3).

use egui::{Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use pier_engine::{Asset, AssetKind, ImageSize};

use crate::i18n::{S, tr};
use crate::icons::{self, Icon};
use crate::shell::{Action, PierShell, fmt_size, fmt_time, kind_name, label_color, reveal_label};
use crate::state::{SortKey, ViewMode};
use crate::theme::Tokens;

/// Space under a thumbnail for the name, the stars and the label band.
const CAPTION_HEIGHT: f32 = 38.0;
const CELL_PAD: f32 = 10.0;
const STAR_SPACING: f32 = 12.0;
const LIST_ROW: f32 = 20.0;
const DETAILS_THUMB: f32 = 96.0;

impl PierShell {
    pub(crate) fn content_panel(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        if self.browser.loading || self.browser.error.is_some() || self.browser.visible.is_empty() {
            let msg = if self.browser.loading {
                S::Loading
            } else if self.browser.error.is_some() {
                S::FolderError
            } else if self.browser.assets.is_empty() {
                S::FolderEmpty
            } else {
                S::NoMatches
            };
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, tr(msg), self.tokens.font(), self.tokens.fg_dim);
            return;
        }
        // A short, wide group (the Filmstrip workspace) shows one row that scrolls sideways.
        if rect.width() > rect.height() * 3.0 && self.ui.view() == ViewMode::Thumbnails {
            self.filmstrip(ui);
            return;
        }
        match self.ui.view() {
            ViewMode::Thumbnails => self.thumbnails(ui),
            ViewMode::Details => self.details(ui),
            ViewMode::List => self.list(ui),
        }
    }

    fn thumbnails(&mut self, ui: &mut Ui) {
        let size = self.ui.thumb_size;
        let avail = ui.available_width() - 12.0;
        let min_cell = size + CELL_PAD * 2.0;
        let cols = ((avail / min_cell).floor() as usize).max(1);
        self.ui.grid_columns = cols;
        let cell_w = (avail / cols as f32).max(min_cell);
        let cell_h = size + CELL_PAD * 2.0 + CAPTION_HEIGHT;
        let n = self.browser.visible.len();
        let rows = n.div_ceil(cols);
        let scroll_to = if std::mem::take(&mut self.ui.scroll_to_anchor) { self.browser.anchor_index() } else { None };
        egui::ScrollArea::vertical().id_salt("content-grid").auto_shrink(false).show_viewport(ui, |ui, viewport| {
            ui.set_min_height(rows as f32 * cell_h + 8.0);
            let origin = ui.min_rect().min + vec2(6.0, 4.0);
            let bg = ui.interact(ui.max_rect(), Id::unique("content-bg"), Sense::click());
            if bg.clicked() {
                self.browser.selection.clear();
            }
            // One screen of rows ahead is requested too, so scrolling finds them decoded.
            let screen_rows = (viewport.height() / cell_h).ceil() as usize;
            let first = (viewport.min.y / cell_h).floor().max(0.0) as usize;
            let last = ((viewport.max.y / cell_h).ceil() as usize).min(rows);
            for row in first..(last + screen_rows).min(rows) {
                for col in 0..cols {
                    let i = row * cols + col;
                    if i >= n {
                        break;
                    }
                    let r = Rect::from_min_size(origin + vec2(col as f32 * cell_w, row as f32 * cell_h), vec2(cell_w, cell_h));
                    if row < last {
                        self.cell(ui, i, r, size);
                    } else if let Some(a) = self.browser.visible_at(i).filter(|a| a.kind.has_thumbnail()).map(|a| a.path.clone()) {
                        self.image(&a, ImageSize::Thumb);
                    }
                    if scroll_to == Some(i) {
                        ui.scroll_to_rect(r, None);
                    }
                }
            }
        });
    }

    fn filmstrip(&mut self, ui: &mut Ui) {
        let h = ui.available_height();
        let size = (h - CELL_PAD * 2.0 - CAPTION_HEIGHT - 14.0).clamp(40.0, 260.0);
        let cell_w = size + CELL_PAD * 2.0;
        let n = self.browser.visible.len();
        self.ui.grid_columns = 1;
        egui::ScrollArea::horizontal().id_salt("content-strip").auto_shrink(false).show_viewport(ui, |ui, viewport| {
            ui.set_min_width(n as f32 * cell_w + 8.0);
            let origin = ui.min_rect().min + vec2(4.0, 2.0);
            let first = (viewport.min.x / cell_w).floor().max(0.0) as usize;
            let last = ((viewport.max.x / cell_w).ceil() as usize).min(n);
            for i in first..last {
                let r = Rect::from_min_size(origin + vec2(i as f32 * cell_w, 0.0), vec2(cell_w, size + CELL_PAD * 2.0 + CAPTION_HEIGHT));
                self.cell(ui, i, r, size);
            }
        });
    }

    /// One thumbnail cell: image, name, stars and label band, with click, double-click and context menu.
    fn cell(&mut self, ui: &mut Ui, index: usize, rect: Rect, size: f32) {
        let Some(asset) = self.browser.visible_at(index).cloned() else { return };
        let t = self.tokens.clone();
        let selected = self.browser.selection.contains(&asset.path);
        let resp = ui.interact(rect.shrink(2.0), Id::unique(("cell", &asset.path)), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::ListItem, true, selected, &asset.name));
        let painter = ui.painter_at(rect);
        if selected {
            painter.rect_filled(rect.shrink(2.0), 4, t.bg_cell_selected);
        } else if resp.hovered() {
            painter.rect_filled(rect.shrink(2.0), 4, t.bg_cell_hover);
        }
        let img_rect = Rect::from_min_size(pos2(rect.center().x - size / 2.0, rect.top() + CELL_PAD), vec2(size, size));
        self.thumb(ui, &asset, img_rect);

        let name_y = img_rect.bottom() + 9.0;
        let job = truncated(&asset.name, &t, rect.width() - 12.0, if selected { t.fg_strong } else { t.fg_text });
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        painter.galley(pos2(rect.center().x - galley.size().x / 2.0, name_y - galley.size().y / 2.0), galley, t.fg_text);

        let stars_center = pos2(rect.center().x, name_y + 16.0);
        if let Some(label) = self.browser.label(&asset.path) {
            let band = Rect::from_center_size(stars_center, vec2((rect.width() - 16.0).min(size + 8.0), 14.0));
            painter.rect_filled(band, 2, label_color(&t, label));
        }
        let rating = self.browser.rating(&asset.path);
        let show_dots = selected || resp.hovered();
        let stars_rect = Rect::from_center_size(stars_center, vec2(STAR_SPACING * 5.0, 14.0));
        if rating < 0 {
            painter.text(stars_center, Align2::CENTER_CENTER, tr(S::Reject), t.font_small(), t.reject);
        } else if !asset.is_folder() {
            paint_stars(&painter, &t, stars_rect, rating, show_dots, self.browser.label(&asset.path).is_some());
        }

        let stars_resp = if asset.is_folder() { None } else { Some(ui.interact(stars_rect, Id::unique(("stars", &asset.path)), Sense::click())) };
        if let Some(sr) = &stars_resp
            && sr.clicked()
            && let Some(p) = sr.interact_pointer_pos()
        {
            let n = (((p.x - stars_rect.left()) / STAR_SPACING).floor() as i8 + 1).clamp(1, 5);
            if !selected {
                self.browser.click_select(&asset.path, false, false);
            }
            self.act(Action::Rating(if rating == n { 0 } else { n }));
        }
        self.cell_interaction(ui, &asset, &resp, selected);
    }

    fn cell_interaction(&mut self, ui: &Ui, asset: &Asset, resp: &Response, selected: bool) {
        let mods = ui.input(|i| i.modifiers);
        if resp.clicked() {
            self.browser.click_select(&asset.path, mods.command, mods.shift);
        }
        if resp.double_clicked() {
            self.browser.click_select(&asset.path, false, false);
            self.act(Action::Open);
        }
        if resp.secondary_clicked() && !selected {
            self.browser.click_select(&asset.path, false, false);
        }
        resp.context_menu(|ui| self.asset_menu(ui));
    }

    /// The context menu of a selected item (docs/ui-design.md §4.3; order provisional).
    fn asset_menu(&mut self, ui: &mut Ui) {
        if ui.button(tr(S::Open)).clicked() {
            self.act(Action::Open);
        }
        ui.menu_button(tr(S::OpenWith), |ui| self.open_with_items(ui));
        if ui.button(tr(S::OpenInCameraRaw)).clicked() {
            self.act(Action::OpenInCameraRaw);
        }
        ui.separator();
        for s in [S::Cut, S::Copy, S::Duplicate, S::MoveToTrash] {
            self.mi_soon(ui, s);
        }
        if ui.button(tr(reveal_label())).clicked() {
            self.act(Action::Reveal);
        }
        self.mi_soon(ui, S::AddToFavorites);
        ui.separator();
        ui.menu_button(tr(S::Rating), |ui| self.rating_items(ui));
        ui.menu_button(tr(S::Label), |ui| self.label_items(ui));
        ui.separator();
        for s in [S::GroupAsStack, S::RotateCw, S::RotateCcw, S::FileInfo, S::BatchRename] {
            self.mi_soon(ui, s);
        }
    }

    /// The image (or icon) of an item, fitted inside `rect`.
    pub(crate) fn thumb(&mut self, ui: &Ui, asset: &Asset, rect: Rect) {
        let t = self.tokens.clone();
        let painter = ui.painter_at(rect.expand(1.0));
        if asset.kind.has_thumbnail() {
            if let Some((tex, size)) = self.image(&asset.path, ImageSize::Thumb) {
                let r = fit(rect, size);
                painter.rect_filled(r.expand(1.0), 0, t.separator);
                painter.image(tex, r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                return;
            }
            if !self.image_failed(&asset.path, ImageSize::Thumb) {
                let r = Rect::from_center_size(rect.center(), rect.size() * vec2(0.8, 0.6));
                painter.rect_filled(r, 2, t.bg_field);
                return;
            }
        }
        file_icon(&painter, &t, rect, asset);
    }

    fn list(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let cols: [(S, Option<SortKey>, f32); 6] = [
            (S::MFilename, Some(SortKey::Name), 0.0),
            (S::SortModified, Some(SortKey::Modified), 150.0),
            (S::SortSize, Some(SortKey::Size), 90.0),
            (S::SortType, Some(SortKey::Type), 120.0),
            (S::SortRating, Some(SortKey::Rating), 80.0),
            (S::SortLabel, Some(SortKey::Label), 90.0),
        ];
        let width = ui.available_width();
        let fixed: f32 = cols.iter().map(|c| c.2).sum();
        let name_w = (width - fixed).max(160.0);
        let widths: Vec<f32> = cols.iter().map(|c| if c.2 == 0.0 { name_w } else { c.2 }).collect();
        let (header, _) = ui.allocate_exact_size(vec2(width, LIST_ROW + 2.0), Sense::hover());
        ui.painter().rect_filled(header, 0, t.bg_section);
        let mut x = header.left();
        for ((label, key, _), w) in cols.iter().zip(&widths) {
            let r = Rect::from_min_size(pos2(x, header.top()), vec2(*w, header.height()));
            let resp = ui.interact(r, Id::unique(("list-header", *label)), Sense::click());
            let active = *key == self.browser.sort;
            let arrow = if active { if self.browser.descending { " ▼" } else { " ▲" } } else { "" };
            ui.painter().text(
                pos2(r.left() + 6.0, r.center().y),
                Align2::LEFT_CENTER,
                format!("{}{arrow}", tr(*label)),
                t.font(),
                if active { t.fg_strong } else { t.fg_dim },
            );
            ui.painter().line_segment([r.right_top(), r.right_bottom()], t.separator_stroke());
            if resp.clicked() {
                if active {
                    self.browser.descending = !self.browser.descending;
                } else {
                    self.browser.sort = *key;
                }
                self.browser.refresh_visible();
            }
            x += w;
        }
        let n = self.browser.visible.len();
        self.ui.grid_columns = 1;
        let scroll_to = if std::mem::take(&mut self.ui.scroll_to_anchor) { self.browser.anchor_index() } else { None };
        egui::ScrollArea::vertical().id_salt("content-list").auto_shrink(false).show_rows(ui, LIST_ROW, n, |ui, range| {
            for i in range {
                let Some(asset) = self.browser.visible_at(i).cloned() else { continue };
                let (row, resp) = ui.allocate_exact_size(vec2(width, LIST_ROW), Sense::click());
                let selected = self.browser.selection.contains(&asset.path);
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Row, true, selected, &asset.name));
                if selected {
                    ui.painter().rect_filled(row, 0, t.bg_cell_selected);
                } else if resp.hovered() {
                    ui.painter().rect_filled(row, 0, t.bg_cell_hover);
                } else if i % 2 == 1 {
                    ui.painter().rect_filled(row, 0, t.bg_content.gamma_multiply(1.15));
                }
                let mut x = row.left();
                let rating = self.browser.rating(&asset.path);
                let cells = [
                    asset.name.clone(),
                    fmt_time(asset.modified),
                    if asset.is_folder() { "—".into() } else { fmt_size(asset.size) },
                    kind_label(&asset),
                    if rating < 0 { tr(S::Reject).into() } else { "★".repeat(rating.max(0) as usize) },
                    self.browser.label(&asset.path).map(|l| tr(l.name()).to_owned()).unwrap_or_default(),
                ];
                for (c, (text, w)) in cells.iter().zip(&widths).enumerate() {
                    let r = Rect::from_min_size(pos2(x, row.top()), vec2(*w, LIST_ROW));
                    let mut text_x = r.left() + 6.0;
                    if c == 0 {
                        self.thumb(ui, &asset, Rect::from_min_size(pos2(r.left() + 4.0, r.top() + 2.0), vec2(16.0, 16.0)));
                        text_x += 20.0;
                    }
                    if c == 5
                        && let Some(l) = self.browser.label(&asset.path)
                    {
                        ui.painter().rect_filled(Rect::from_min_size(pos2(r.left() + 6.0, r.center().y - 5.0), vec2(10.0, 10.0)), 2, label_color(&t, l));
                        text_x += 14.0;
                    }
                    let job = truncated(text, &t, r.right() - text_x - 4.0, if selected { t.fg_strong } else { t.fg_text });
                    let g = ui.fonts_mut(|f| f.layout_job(job));
                    ui.painter_at(r).galley(pos2(text_x, r.center().y - g.size().y / 2.0), g, t.fg_text);
                    x += w;
                }
                if scroll_to == Some(i) {
                    ui.scroll_to_rect(row, None);
                }
                self.cell_interaction(ui, &asset, &resp, selected);
            }
        });
    }

    fn details(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let row_h = DETAILS_THUMB + 12.0;
        let n = self.browser.visible.len();
        let width = ui.available_width();
        self.ui.grid_columns = 1;
        let scroll_to = if std::mem::take(&mut self.ui.scroll_to_anchor) { self.browser.anchor_index() } else { None };
        egui::ScrollArea::vertical().id_salt("content-details").auto_shrink(false).show_rows(ui, row_h, n, |ui, range| {
            for i in range {
                let Some(asset) = self.browser.visible_at(i).cloned() else { continue };
                let (row, resp) = ui.allocate_exact_size(vec2(width, row_h), Sense::click());
                let selected = self.browser.selection.contains(&asset.path);
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Row, true, selected, &asset.name));
                if selected {
                    ui.painter().rect_filled(row.shrink(1.0), 3, t.bg_cell_selected);
                } else if resp.hovered() {
                    ui.painter().rect_filled(row.shrink(1.0), 3, t.bg_cell_hover);
                }
                self.thumb(ui, &asset, Rect::from_min_size(row.min + vec2(10.0, 6.0), vec2(DETAILS_THUMB, DETAILS_THUMB)));
                let x0 = row.left() + DETAILS_THUMB + 24.0;
                let col_w = ((row.right() - x0) / 2.0).max(120.0);
                let rating = self.browser.rating(&asset.path);
                let left = [
                    (asset.name.clone(), t.fg_strong),
                    (format!("{}: {}", tr(S::MCreated), fmt_time(asset.created)), t.fg_text),
                    (format!("{}: {}", tr(S::MModified), fmt_time(asset.modified)), t.fg_text),
                    (if asset.is_folder() { String::new() } else { fmt_size(asset.size) }, t.fg_text),
                ];
                let right = [
                    (kind_label(&asset), t.fg_text),
                    (if rating < 0 { tr(S::Reject).into() } else { "★".repeat(rating.max(0) as usize) }, t.star),
                    (self.browser.label(&asset.path).map(|l| tr(l.name()).to_owned()).unwrap_or_default(), t.fg_text),
                ];
                for (col, lines) in [(0.0, left.as_slice()), (col_w, right.as_slice())] {
                    for (k, (text, color)) in lines.iter().enumerate() {
                        let job = truncated(text, &t, col_w - 12.0, *color);
                        let g = ui.fonts_mut(|f| f.layout_job(job));
                        ui.painter_at(row).galley(pos2(x0 + col, row.top() + 12.0 + k as f32 * 18.0), g, *color);
                    }
                }
                if scroll_to == Some(i) {
                    ui.scroll_to_rect(row, None);
                }
                self.cell_interaction(ui, &asset, &resp, selected);
            }
        });
    }
}

pub(crate) fn kind_label(a: &Asset) -> String {
    if a.ext.is_empty() { tr(kind_name(a.kind)).to_owned() } else { format!("{} ({})", tr(kind_name(a.kind)), a.ext.to_uppercase()) }
}

pub(crate) fn truncated(text: &str, t: &Tokens, width: f32, color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), t.font(), color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(10.0));
    job
}

/// The largest rect of `size`'s aspect that fits in `outer`, centred.
pub(crate) fn fit(outer: Rect, size: egui::Vec2) -> Rect {
    if size.x <= 0.0 || size.y <= 0.0 {
        return outer;
    }
    let scale = (outer.width() / size.x).min(outer.height() / size.y);
    Rect::from_center_size(outer.center(), size * scale)
}

fn paint_stars(p: &egui::Painter, t: &Tokens, rect: Rect, rating: i8, dots: bool, on_label: bool) {
    let color = if on_label { Color32::from_gray(20) } else { t.star };
    for i in 0..5 {
        let c = pos2(rect.left() + STAR_SPACING * (i as f32 + 0.5), rect.center().y);
        if (i as i8) < rating {
            icons::star(p, c, 5.0, color);
        } else if dots {
            p.circle_filled(c, 1.3, if on_label { color } else { t.star_off });
        }
    }
}

/// A document icon with the file's extension, or the folder / media symbols.
pub(crate) fn file_icon(p: &egui::Painter, t: &Tokens, rect: Rect, asset: &Asset) {
    let side = rect.width().min(rect.height());
    if asset.kind == AssetKind::Folder {
        let r = Rect::from_center_size(rect.center(), vec2(side * 0.78, side * 0.6));
        icons::paint(p, r, Icon::Folder, Color32::from_rgb(0x5a, 0x8f, 0xc8));
        return;
    }
    let page = Rect::from_center_size(rect.center(), vec2(side * 0.6, side * 0.78));
    let fold = side * 0.14;
    let pts: Vec<Pos2> =
        vec![page.left_top(), pos2(page.right() - fold, page.top()), pos2(page.right(), page.top() + fold), page.right_bottom(), page.left_bottom()];
    let fill = if t.brightness.is_dark() { Color32::from_gray(0xd8) } else { Color32::WHITE };
    p.add(egui::Shape::convex_polygon(pts, fill, Stroke::new(1.0, t.separator)));
    let symbol = match asset.kind {
        AssetKind::Video => Some(Icon::Play),
        AssetKind::Audio => Some(Icon::Music),
        AssetKind::Image | AssetKind::Raw | AssetKind::Layered => Some(Icon::Picture),
        _ => None,
    };
    if let Some(icon) = symbol {
        icons::paint(p, Rect::from_center_size(page.center() - vec2(0.0, side * 0.08), egui::Vec2::splat(side * 0.28)), icon, Color32::from_gray(0x70));
    }
    if !asset.ext.is_empty() && side >= 32.0 {
        let badge = Rect::from_center_size(pos2(page.center().x, page.bottom() - side * 0.16), vec2(page.width() * 0.8, (side * 0.13).max(10.0)));
        p.rect_filled(badge, CornerRadius::same(2), t.accent);
        let font = egui::FontId::proportional((side * 0.09).clamp(8.0, 13.0));
        p.text(badge.center(), Align2::CENTER_CENTER, asset.ext.to_uppercase(), font, Color32::WHITE);
    }
}
