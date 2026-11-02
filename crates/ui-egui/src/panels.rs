//! The side panels (docs/ui-design.md §4). Content lives in content.rs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, Id, Rect, RichText, Sense, Ui, pos2, vec2};
use pier_engine::fs::Place;
use pier_engine::{AssetKind, ImageSize};

use crate::content::{file_icon, fit, kind_label, truncated};
use crate::i18n::{S, tr};
use crate::icons::{self, Icon};
use crate::shell::{Action, PierShell, fmt_size, fmt_time, label_color};
use crate::state::{Label, date_key};
use crate::theme::Tokens;

pub(crate) const ROW: f32 = 20.0;
const TREE_INDENT: f32 = 14.0;
/// Folders panel depth limit (a symbolic-link loop can't recurse forever).
const MAX_TREE_DEPTH: usize = 48;
/// Dates listed under Filter › Date Modified, newest first.
const MAX_FILTER_DATES: usize = 30;
/// Items the Preview panel compares at once (Bridge shows up to nine).
const MAX_PREVIEW_ITEMS: usize = 9;

pub(crate) fn place_name(p: Place) -> S {
    match p {
        Place::Computer => S::Computer,
        Place::Home => S::Home,
        Place::Desktop => S::Desktop,
        Place::Documents => S::Documents,
        Place::Pictures => S::Pictures,
        Place::Downloads => S::Downloads,
    }
}

fn place_icon(p: Place) -> Icon {
    match p {
        Place::Computer => Icon::Computer,
        Place::Home => Icon::Home,
        Place::Desktop => Icon::Desktop,
        Place::Documents => Icon::Document,
        Place::Pictures => Icon::Picture,
        Place::Downloads => Icon::Download,
    }
}

/// The note shown on panels whose behaviour isn't built yet.
pub(crate) fn soon_banner(ui: &mut Ui, t: &Tokens) {
    egui::Frame::NONE.fill(t.bg_section).stroke(egui::Stroke::new(1.0, t.accent.gamma_multiply(0.6))).corner_radius(3).inner_margin(6).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(tr(S::PanelSoon)).color(t.fg_dim).small());
    });
}

/// A full-width row with a hover/selected background; returns its rect and response.
pub(crate) fn row(ui: &mut Ui, t: &Tokens, selected: bool, id: impl std::hash::Hash + std::fmt::Debug, label: &str) -> (Rect, egui::Response) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
    let resp = ui.interact(rect, Id::unique(id), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::ListItem, true, selected, label));
    if selected {
        ui.painter().rect_filled(rect, 0, t.bg_cell_selected);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 0, t.bg_cell_hover);
    }
    (rect, resp)
}

/// A bar of small buttons along the bottom of a panel.
fn bottom_bar(ui: &mut Ui, t: &Tokens, add: impl FnOnce(&mut Ui)) {
    let rect = ui.max_rect();
    let bar = Rect::from_min_max(pos2(rect.left(), rect.bottom() - 24.0), rect.max);
    ui.painter().rect_filled(bar, 0, t.bg_tab_strip);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(bar.shrink2(vec2(4.0, 2.0))).layout(egui::Layout::right_to_left(egui::Align::Center)));
    add(&mut child);
}

fn body_rect(ui: &Ui, bottom: bool) -> Rect {
    let r = ui.max_rect();
    if bottom { Rect::from_min_max(r.min, pos2(r.right(), r.bottom() - 24.0)) } else { r }
}

impl PierShell {
    pub(crate) fn favorites_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.add_space(4.0);
        for (place, path) in pier_engine::fs::default_places() {
            let (rect, resp) = row(ui, &t, path == self.browser.path, ("fav", &path), tr(place_name(place)));
            icons::paint(ui.painter(), Rect::from_min_size(pos2(rect.left() + 10.0, rect.top() + 3.0), vec2(14.0, 14.0)), place_icon(place), t.fg_icon);
            ui.painter().text(pos2(rect.left() + 32.0, rect.center().y), Align2::LEFT_CENTER, tr(place_name(place)), t.font(), t.fg_text);
            if resp.clicked() {
                self.act(Action::GoTo(path));
            }
        }
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(tr(S::DragFavoritesHint)).color(t.fg_disabled).small());
        });
    }

    pub(crate) fn folders_panel(&mut self, ui: &mut Ui) {
        let mut roots: Vec<(String, PathBuf)> = Vec::new();
        if let Some(h) = pier_engine::fs::home_dir() {
            roots.push((tr(S::Home).to_owned(), h));
        }
        roots.push((tr(S::Computer).to_owned(), pier_engine::fs::computer_root()));
        egui::ScrollArea::vertical().id_salt("folders").auto_shrink(false).show(ui, |ui| {
            ui.add_space(2.0);
            for (name, path) in roots {
                self.tree_node(ui, &path, Some(&name), 0);
            }
        });
    }

    fn tree_node(&mut self, ui: &mut Ui, path: &Path, name: Option<&str>, depth: usize) {
        if depth > MAX_TREE_DEPTH {
            return;
        }
        let t = self.tokens.clone();
        let expanded = self.ui.tree_expanded.contains(path);
        let children = self.subfolders.get(path).cloned().flatten();
        let label = name.map(str::to_owned).unwrap_or_else(|| pier_engine::fs::display_name(path));
        let (rect, resp) = row(ui, &t, path == self.browser.path, ("tree", path, depth), &label);
        let x = rect.left() + 6.0 + depth as f32 * TREE_INDENT;
        let tri = Rect::from_min_size(pos2(x, rect.top() + 5.0), vec2(10.0, 10.0));
        let has_children = !matches!(&children, Some(Ok(c)) if c.is_empty());
        if has_children {
            icons::paint(ui.painter(), tri, if expanded { Icon::TriangleDown } else { Icon::TriangleRight }, t.fg_dim);
        }
        icons::paint(ui.painter(), Rect::from_min_size(pos2(x + 13.0, rect.top() + 3.0), vec2(15.0, 14.0)), Icon::Folder, Color32::from_rgb(0x5a, 0x8f, 0xc8));
        let job = truncated(&label, &t, rect.right() - x - 36.0, t.fg_text);
        let g = ui.fonts_mut(|f| f.layout_job(job));
        ui.painter().galley(pos2(x + 32.0, rect.center().y - g.size().y / 2.0), g, t.fg_text);
        let tri_hit = ui.interact(tri.expand(4.0), Id::unique(("tree-tri", path, depth)), Sense::click());
        if tri_hit.clicked() {
            if expanded {
                self.ui.tree_expanded.remove(path);
            } else {
                self.ui.tree_expanded.insert(path.to_path_buf());
                self.want_subfolders(path);
            }
        } else if resp.clicked() {
            self.act(Action::GoTo(path.to_path_buf()));
        }
        if expanded {
            self.want_subfolders(path);
            match children {
                Some(Ok(kids)) => {
                    for kid in kids {
                        self.tree_node(ui, &kid, None, depth + 1);
                    }
                }
                None => {
                    ui.horizontal(|ui| {
                        ui.add_space(x + 32.0 - ui.min_rect().left());
                        ui.label(RichText::new(tr(S::Loading)).color(t.fg_disabled).small());
                    });
                }
                Some(Err(_)) => {}
            }
        }
    }

    pub(crate) fn filter_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let base: Vec<_> = self.browser.assets.iter().filter(|a| self.browser.passes_base(a)).cloned().collect();
        let mut labels: BTreeMap<Option<Label>, usize> = BTreeMap::new();
        let mut ratings: BTreeMap<i8, usize> = BTreeMap::new();
        let mut kinds: BTreeMap<AssetKind, usize> = BTreeMap::new();
        let mut dates: BTreeMap<String, usize> = BTreeMap::new();
        for a in &base {
            *labels.entry(self.browser.label(&a.path)).or_default() += 1;
            *ratings.entry(self.browser.rating(&a.path)).or_default() += 1;
            *kinds.entry(a.kind).or_default() += 1;
            if let Some(d) = date_key(a) {
                *dates.entry(d).or_default() += 1;
            }
        }
        let body = body_rect(ui, true);
        let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body));
        egui::ScrollArea::vertical().id_salt("filter").auto_shrink(false).show(&mut body_ui, |ui| {
            ui.add_space(2.0);
            if self.section(ui, S::FLabels) {
                for (l, n) in &labels {
                    let text = l.map_or(tr(S::NoLabel), |l| tr(l.name()));
                    let on = self.browser.filter.labels.contains(l);
                    if self.filter_row(ui, &t, ("fl", *l), text, *n, on, l.map(|l| label_color(&t, l))) {
                        toggle(&mut self.browser.filter.labels, *l);
                        self.browser.refresh_visible();
                    }
                }
            }
            if self.section(ui, S::FRatings) {
                for (r, n) in ratings.iter().rev() {
                    let text = match *r {
                        0 => tr(S::NoRating).to_owned(),
                        r if r < 0 => tr(S::Reject).to_owned(),
                        r => "★".repeat(r as usize),
                    };
                    let on = self.browser.filter.ratings.contains(r);
                    if self.filter_row(ui, &t, ("fr", *r), &text, *n, on, None) {
                        toggle(&mut self.browser.filter.ratings, *r);
                        self.browser.refresh_visible();
                    }
                }
            }
            if self.section(ui, S::FFileType) {
                for (k, n) in &kinds {
                    let on = self.browser.filter.kinds.contains(k);
                    if self.filter_row(ui, &t, ("fk", *k), tr(crate::shell::kind_name(*k)), *n, on, None) {
                        toggle(&mut self.browser.filter.kinds, *k);
                        self.browser.refresh_visible();
                    }
                }
            }
            if self.section(ui, S::FDateModified) {
                for (d, n) in dates.iter().rev().take(MAX_FILTER_DATES) {
                    let on = self.browser.filter.dates.contains(d);
                    if self.filter_row(ui, &t, ("fd", d.as_str()), d, *n, on, None) {
                        toggle(&mut self.browser.filter.dates, d.clone());
                        self.browser.refresh_visible();
                    }
                }
            }
        });
        bottom_bar(ui, &t, |ui| {
            if icons::button(ui, &t, Icon::Clear, 20.0, tr(S::FClear), !self.browser.filter.is_empty()).clicked() {
                self.browser.filter = crate::state::Filter::default();
                self.browser.refresh_visible();
            }
            if icons::button(ui, &t, Icon::Pin, 20.0, tr(S::FKeepFilter), true).clicked() {
                self.soon(S::FKeepFilter);
            }
        });
    }

    /// A collapsible section header (Filter and Metadata panels). Returns whether it is open.
    pub(crate) fn section(&mut self, ui: &mut Ui, title: S) -> bool {
        let t = self.tokens.clone();
        let open = !self.ui.collapsed_sections.contains(&title);
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW + 1.0), Sense::hover());
        let resp = ui.interact(rect, Id::unique(("section", title)), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr(title)));
        ui.painter().rect_filled(rect.shrink2(vec2(0.0, 0.5)), 0, t.bg_section);
        icons::paint(
            ui.painter(),
            Rect::from_min_size(pos2(rect.left() + 6.0, rect.top() + 6.0), vec2(9.0, 9.0)),
            if open { Icon::TriangleDown } else { Icon::TriangleRight },
            t.fg_dim,
        );
        ui.painter().text(pos2(rect.left() + 20.0, rect.center().y), Align2::LEFT_CENTER, tr(title), t.font(), t.fg_strong);
        if resp.clicked() {
            if open {
                self.ui.collapsed_sections.insert(title);
            } else {
                self.ui.collapsed_sections.remove(&title);
            }
        }
        open
    }

    #[allow(clippy::too_many_arguments)]
    fn filter_row(
        &self,
        ui: &mut Ui,
        t: &Tokens,
        id: impl std::hash::Hash + std::fmt::Debug,
        text: &str,
        count: usize,
        on: bool,
        swatch: Option<Color32>,
    ) -> bool {
        let (rect, resp) = row(ui, t, on, id, text);
        let check = Rect::from_min_size(pos2(rect.left() + 14.0, rect.top() + 5.0), vec2(10.0, 10.0));
        if on {
            icons::paint(ui.painter(), check.expand(2.0), Icon::Check, t.accent);
        }
        let mut x = rect.left() + 32.0;
        if let Some(c) = swatch {
            ui.painter().rect_filled(Rect::from_min_size(pos2(x, rect.top() + 5.0), vec2(10.0, 10.0)), 2, c);
            x += 16.0;
        }
        ui.painter().text(pos2(x, rect.center().y), Align2::LEFT_CENTER, text, t.font(), if on { t.fg_strong } else { t.fg_text });
        ui.painter().text(pos2(rect.right() - 10.0, rect.center().y), Align2::RIGHT_CENTER, count.to_string(), t.font(), t.fg_dim);
        resp.clicked()
    }

    pub(crate) fn collections_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let body = body_rect(ui, true);
        let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body.shrink(6.0)));
        soon_banner(&mut body_ui, &t);
        body_ui.add_space(8.0);
        body_ui.label(RichText::new(tr(S::CNone)).color(t.fg_disabled));
        bottom_bar(ui, &t, |ui| {
            for (icon, s) in [(Icon::Trash, S::TipDelete), (Icon::Collection, S::CNew), (Icon::Refine, S::CEditSmart), (Icon::SmartCollection, S::CNewSmart)] {
                if icons::button(ui, &t, icon, 20.0, tr(s), true).clicked() {
                    self.soon(s);
                }
            }
        });
    }

    pub(crate) fn export_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(ui.max_rect().shrink(6.0)));
        soon_banner(&mut inner, &t);
        inner.add_space(6.0);
        let (rect, resp) = row(&mut inner, &t, false, "export-preset", tr(S::ESaveToDisk));
        icons::paint(inner.painter(), Rect::from_min_size(pos2(rect.left() + 6.0, rect.top() + 3.0), vec2(14.0, 14.0)), Icon::Download, t.fg_icon);
        inner.painter().text(pos2(rect.left() + 28.0, rect.center().y), Align2::LEFT_CENTER, tr(S::ESaveToDisk), t.font(), t.fg_text);
        if resp.clicked() {
            self.soon(S::PExport);
        }
        let (rect, resp) = row(&mut inner, &t, false, "export-add", tr(S::EAddPreset));
        icons::paint(inner.painter(), Rect::from_min_size(pos2(rect.left() + 6.0, rect.top() + 3.0), vec2(14.0, 14.0)), Icon::Plus, t.fg_dim);
        inner.painter().text(pos2(rect.left() + 28.0, rect.center().y), Align2::LEFT_CENTER, tr(S::EAddPreset), t.font(), t.fg_dim);
        if resp.clicked() {
            self.soon(S::EAddPreset);
        }
    }

    pub(crate) fn preview_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let items: Vec<_> = self.browser.selected_assets().into_iter().take(MAX_PREVIEW_ITEMS).cloned().collect();
        let area = ui.max_rect().shrink(10.0);
        if items.is_empty() || area.width() < 8.0 || area.height() < 8.0 {
            return;
        }
        let cols = (items.len() as f32).sqrt().ceil() as usize;
        let rows = items.len().div_ceil(cols);
        let cell = vec2(area.width() / cols as f32, area.height() / rows as f32);
        for (i, a) in items.iter().enumerate() {
            let r = Rect::from_min_size(area.min + vec2((i % cols) as f32 * cell.x, (i / cols) as f32 * cell.y), cell).shrink(4.0);
            let resp = ui.interact(r, Id::unique(("preview", &a.path)), Sense::click());
            if a.kind.has_thumbnail() {
                let img = self.image(&a.path, ImageSize::Preview).or_else(|| self.image(&a.path, ImageSize::Thumb));
                if let Some((tex, size)) = img {
                    ui.painter().image(tex, fit(r, size), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                    if resp.clicked() {
                        self.soon(S::PPreview);
                    }
                    continue;
                }
                if !self.image_failed(&a.path, ImageSize::Thumb) {
                    continue;
                }
            }
            file_icon(ui.painter(), &t, Rect::from_center_size(r.center(), egui::Vec2::splat(r.width().min(r.height()).min(160.0))), a);
        }
    }

    pub(crate) fn metadata_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let selected: Vec<_> = self.browser.selected_assets().into_iter().cloned().collect();
        let Some(asset) = selected.first().cloned() else {
            ui.painter().text(ui.max_rect().center(), Align2::CENTER_CENTER, tr(S::MNoSelection), t.font(), t.fg_dim);
            return;
        };
        egui::ScrollArea::vertical().id_salt("metadata").auto_shrink(false).show(ui, |ui| {
            ui.add_space(6.0);
            if selected.len() > 1 {
                let total: u64 = selected.iter().map(|a| a.size).sum();
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(tr(S::MMultiple).replace("{n}", &selected.len().to_string())).color(t.fg_strong));
                    ui.label(RichText::new(fmt_size(total)).color(t.fg_dim));
                });
                ui.add_space(6.0);
                note(ui, &t, tr(S::MarksSessionOnly));
                return;
            }
            let meta = if asset.is_folder() { None } else { self.want_meta(&asset.path).cloned() };
            let m = meta.unwrap_or_default();
            placard(ui, &t, &m, &asset);
            ui.add_space(6.0);
            let rating = self.browser.rating(&asset.path);
            if self.section(ui, S::MFileProperties) {
                let dims = m.dimensions.map(|(w, h)| format!("{w} × {h}"));
                let rating_text = if rating < 0 { tr(S::Reject).to_owned() } else { "★".repeat(rating.max(0) as usize) };
                let label = self.browser.label(&asset.path).map(|l| tr(l.name()).to_owned());
                let parent = asset.path.parent().map(|p| p.display().to_string());
                kv(ui, &t, S::MFilename, Some(asset.name.clone()));
                kv(ui, &t, S::MDocumentType, Some(kind_label(&asset)));
                kv(ui, &t, S::MCreated, Some(fmt_time(asset.created)));
                kv(ui, &t, S::MModified, Some(fmt_time(asset.modified)));
                kv(ui, &t, S::MFileSize, (!asset.is_folder()).then(|| fmt_size(asset.size)));
                kv(ui, &t, S::MDimensions, dims);
                kv(ui, &t, S::Label, label);
                kv(ui, &t, S::Rating, Some(rating_text).filter(|s| !s.is_empty()));
                kv(ui, &t, S::MLocation, parent);
            }
            if self.section(ui, S::MIptcCore) {
                kv(ui, &t, S::MCreator, m.artist.clone());
                kv(ui, &t, S::MCopyright, m.copyright.clone());
                note(ui, &t, tr(S::MEditSoon));
            }
            if self.section(ui, S::MCameraData) {
                for (k, v) in [
                    (S::MMake, &m.make),
                    (S::MModel, &m.model),
                    (S::MLens, &m.lens),
                    (S::MDateOriginal, &m.date_time_original),
                    (S::MExposure, &m.exposure),
                    (S::MFNumber, &m.f_number),
                    (S::MIso, &m.iso),
                    (S::MFocal, &m.focal_length),
                    (S::MBias, &m.exposure_bias),
                    (S::MWhiteBalance, &m.white_balance),
                    (S::MFlash, &m.flash),
                    (S::MOrientation, &m.orientation),
                    (S::MResolution, &m.resolution),
                    (S::MColorSpace, &m.color_space),
                    (S::MSoftware, &m.software),
                ] {
                    kv(ui, &t, k, v.clone());
                }
            }
            if (m.gps_latitude.is_some() || m.gps_longitude.is_some()) && self.section(ui, S::MGps) {
                kv(ui, &t, S::MLatitude, m.gps_latitude.clone());
                kv(ui, &t, S::MLongitude, m.gps_longitude.clone());
            }
            ui.add_space(8.0);
            note(ui, &t, tr(S::MarksSessionOnly));
        });
    }

    pub(crate) fn keywords_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let body = body_rect(ui, true);
        let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(body.shrink(6.0)));
        soon_banner(&mut inner, &t);
        inner.add_space(4.0);
        inner.label(RichText::new(tr(S::KAssigned)).color(t.fg_dim));
        inner.add_space(4.0);
        for s in [S::KEvents, S::KPeople, S::KPlaces, S::KOther] {
            let (rect, _) = row(&mut inner, &t, false, ("kw", s), tr(s));
            icons::paint(inner.painter(), Rect::from_min_size(pos2(rect.left() + 4.0, rect.top() + 6.0), vec2(9.0, 9.0)), Icon::TriangleRight, t.fg_dim);
            inner.painter().rect_stroke(
                Rect::from_min_size(pos2(rect.left() + 20.0, rect.top() + 4.0), vec2(12.0, 12.0)),
                2,
                egui::Stroke::new(1.0, t.fg_disabled),
                egui::StrokeKind::Inside,
            );
            inner.painter().text(pos2(rect.left() + 40.0, rect.center().y), Align2::LEFT_CENTER, tr(s), t.font(), t.fg_text);
        }
        bottom_bar(ui, &t, |ui| {
            for (icon, s) in [(Icon::Trash, S::KDelete), (Icon::PlusSub, S::KNewSub), (Icon::Plus, S::KNew)] {
                if icons::button(ui, &t, icon, 20.0, tr(s), true).clicked() {
                    self.soon(s);
                }
            }
            let mut q = String::new();
            ui.add_enabled(false, egui::TextEdit::singleline(&mut q).hint_text(tr(S::QuickSearch)).desired_width(ui.available_width() - 8.0));
        });
    }

    pub(crate) fn output_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        egui::ScrollArea::vertical().id_salt("output").auto_shrink(false).show(ui, |ui| {
            ui.add_space(6.0);
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(6, 0)).show(ui, |ui| soon_banner(ui, &t));
            ui.add_space(6.0);
            ui.add_enabled_ui(false, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(tr(S::OTemplate));
                    let _ = ui.button("4 × 5 ▼");
                });
            });
            for s in [S::ODocument, S::OLayout, S::OOverlays, S::OHeader, S::OFooter, S::OWatermark] {
                if self.section(ui, s) {
                    ui.add_enabled_ui(false, |ui| {
                        let (mut a, mut b) = (4u32, 5u32);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            match s {
                                S::ODocument => {
                                    ui.label(tr(S::OPaper));
                                    let _ = ui.button("A4 ▼");
                                }
                                S::OLayout => {
                                    let l = ui.label(tr(S::OColumns));
                                    ui.add(egui::DragValue::new(&mut a)).labelled_by(l.id);
                                    let l = ui.label(tr(S::ORows));
                                    ui.add(egui::DragValue::new(&mut b)).labelled_by(l.id);
                                }
                                _ => {
                                    ui.label(RichText::new(tr(S::ComingSoon)).color(t.fg_disabled));
                                }
                            }
                        });
                    });
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.button(tr(S::ORefresh)).clicked() {
                    self.soon(S::ORefresh);
                }
                if ui.button(tr(S::OExportPdf)).clicked() {
                    self.soon(S::OExportPdf);
                }
            });
        });
    }

    pub(crate) fn output_preview_panel(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        let area = ui.max_rect().shrink(24.0);
        // A4 portrait page with a 4 × 5 contact sheet of the first images.
        let page = fit(area, vec2(210.0, 297.0));
        ui.painter().rect_filled(page.translate(vec2(3.0, 3.0)), 0, Color32::from_black_alpha(80));
        ui.painter().rect_filled(page, 0, Color32::WHITE);
        let grid = page.shrink(page.width() * 0.06);
        let cell = vec2(grid.width() / 4.0, grid.height() / 5.0);
        let images: Vec<_> = self.browser.visible_assets().filter(|a| a.kind.has_thumbnail()).take(20).map(|a| a.path.clone()).collect();
        for (i, p) in images.iter().enumerate() {
            let r = Rect::from_min_size(grid.min + vec2((i % 4) as f32 * cell.x, (i / 4) as f32 * cell.y), cell).shrink(cell.x * 0.06);
            if let Some((tex, size)) = self.image(p, ImageSize::Thumb) {
                ui.painter().image(tex, fit(r, size), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
        }
        ui.painter().text(pos2(page.center().x, page.bottom() + 12.0), Align2::CENTER_TOP, tr(S::PanelSoon), t.font_small(), t.fg_dim);
    }
}

fn toggle<T: Ord>(set: &mut std::collections::BTreeSet<T>, v: T) {
    if !set.remove(&v) {
        set.insert(v);
    }
}

fn kv(ui: &mut Ui, t: &Tokens, key: S, value: Option<String>) {
    let Some(value) = value.filter(|v| !v.is_empty()) else { return };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    let key_w = (rect.width() * 0.46).clamp(90.0, 190.0);
    let kj = truncated(tr(key), t, key_w - 18.0, t.fg_dim);
    let vj = truncated(&value, t, rect.width() - key_w - 8.0, t.fg_text);
    let (kg, vg) = ui.fonts_mut(|f| (f.layout_job(kj), f.layout_job(vj)));
    ui.painter().galley(pos2(rect.left() + 14.0, rect.center().y - kg.size().y / 2.0), kg, t.fg_dim);
    ui.painter().galley(pos2(rect.left() + key_w, rect.center().y - vg.size().y / 2.0), vg, t.fg_text);
}

fn note(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space(10.0);
        ui.set_max_width(ui.available_width() - 10.0);
        ui.label(RichText::new(text).color(t.fg_disabled).small());
    });
}

/// The camera-style summary at the top of the Metadata panel (docs/ui-design.md §4.5).
fn placard(ui: &mut Ui, t: &Tokens, m: &pier_engine::meta::MetaSummary, a: &pier_engine::Asset) {
    let (outer, _) = ui.allocate_exact_size(vec2(ui.available_width(), 66.0), Sense::hover());
    let r = outer.shrink2(vec2(8.0, 2.0));
    ui.painter().rect_filled(r, 4, t.bg_placard);
    let big = egui::FontId::proportional(t.font_ui + 1.5);
    let dash = || "--".to_owned();
    let left = [
        [m.f_number.clone().unwrap_or_else(dash), m.exposure.clone().unwrap_or_else(dash)],
        [m.exposure_bias.clone().unwrap_or_else(dash), m.iso.clone().unwrap_or_else(dash)],
        [m.focal_length.clone().unwrap_or_else(dash), m.white_balance.clone().unwrap_or_else(dash)],
    ];
    let half = r.width() / 2.0;
    for (row, cells) in left.iter().enumerate() {
        for (col, text) in cells.iter().enumerate() {
            let pos = pos2(r.left() + 10.0 + col as f32 * (half - 10.0) / 2.0, r.top() + 12.0 + row as f32 * 20.0);
            let job = truncated(text, t, (half - 20.0) / 2.0, t.fg_strong);
            let mut job = job;
            if let Some(s) = job.sections.first_mut() {
                s.format.font_id = big.clone();
            }
            let g = ui.fonts_mut(|f| f.layout_job(job));
            ui.painter().galley(pos2(pos.x, pos.y - g.size().y / 2.0), g, t.fg_strong);
        }
    }
    ui.painter().line_segment([pos2(r.center().x, r.top() + 8.0), pos2(r.center().x, r.bottom() - 8.0)], t.separator_stroke());
    let right = [
        m.dimensions.map(|(w, h)| format!("{w} × {h}")).unwrap_or_else(dash),
        if a.is_folder() { dash() } else { fmt_size(a.size) },
        m.resolution.clone().map(|r| format!("{r} ppi")).unwrap_or_else(dash),
        m.color_space.clone().unwrap_or_else(|| a.ext.to_uppercase()),
    ];
    for (i, text) in right.iter().enumerate() {
        let job = truncated(text, t, half - 20.0, t.fg_text);
        let g = ui.fonts_mut(|f| f.layout_job(job));
        ui.painter().galley(pos2(r.center().x + 12.0, r.top() + 5.0 + i as f32 * 14.5), g, t.fg_text);
    }
}
