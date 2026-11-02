//! The menu bar, application bar, path bar and status bar (docs/ui-design.md §1–§3, docs/command-catalog.md).

use egui::{Align, Button, Key, KeyboardShortcut, Layout, RichText, Sense, Ui, Vec2, vec2};
use pier_engine::craft;

use crate::i18n::{S, tr};
use crate::icons::{self, Icon};
use crate::shell::{Action, PierShell, mod_key, reject_shortcut};
use crate::state::{Label, QuickFilter, SortKey, THUMB_MAX, THUMB_MIN, ViewMode};
use crate::workspace::Workspace;

const BAR_ICON: f32 = 26.0;
const SMALL_ICON: f32 = 20.0;

impl PierShell {
    fn mi(&mut self, ui: &mut Ui, label: S, shortcut: Option<KeyboardShortcut>, action: Action) {
        let mut b = Button::new(tr(label));
        if let Some(sc) = shortcut {
            b = b.shortcut_text(ui.ctx().format_shortcut(&sc));
        }
        if ui.add(b).clicked() {
            self.act(action);
        }
    }

    fn mi_text(&mut self, ui: &mut Ui, label: &str, action: Action) {
        if ui.button(label).clicked() {
            self.act(action);
        }
    }

    /// A menu item for something not built yet: shown with "Coming soon"; clicking says so.
    pub(crate) fn mi_soon(&mut self, ui: &mut Ui, label: S) {
        let b = Button::new(tr(label)).shortcut_text(RichText::new(tr(S::ComingSoon)).color(self.tokens.fg_disabled).small());
        if ui.add(b).clicked() {
            self.soon(label);
        }
    }

    pub(crate) fn open_with_items(&mut self, ui: &mut Ui) {
        self.mi(ui, S::DefaultApp, None, Action::OpenWith(None));
        ui.separator();
        for app in craft::ALL {
            let installed = self.craft_apps.get(app.id).is_some_and(Option::is_some);
            let text = if installed { app.name.to_owned() } else { format!("{}  ({})", app.name, tr(S::NotInstalled)) };
            self.mi_text(ui, &text, Action::OpenWith(Some(app)));
        }
    }

    pub(crate) fn rating_items(&mut self, ui: &mut Ui) {
        self.mi(ui, S::NoRating, Some(mod_key(Key::Num0)), Action::Rating(0));
        self.mi(ui, S::Reject, Some(reject_shortcut()), Action::Rating(-1));
        let keys = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5];
        for (n, key) in (1..=5).zip(keys) {
            let b = Button::new("★".repeat(n as usize)).shortcut_text(ui.ctx().format_shortcut(&mod_key(key)));
            if ui.add(b).clicked() {
                self.act(Action::Rating(n));
            }
        }
        ui.separator();
        self.mi(ui, S::DecreaseRating, Some(mod_key(Key::Comma)), Action::RatingStep(-1));
        self.mi(ui, S::IncreaseRating, Some(mod_key(Key::Period)), Action::RatingStep(1));
    }

    pub(crate) fn label_items(&mut self, ui: &mut Ui) {
        self.mi(ui, S::NoLabel, None, Action::Label(None));
        let keys = [Some(Key::Num6), Some(Key::Num7), Some(Key::Num8), Some(Key::Num9), None];
        for (l, key) in Label::ALL.into_iter().zip(keys) {
            self.mi(ui, l.name(), key.map(mod_key), Action::Label(Some(l)));
        }
    }

    pub(crate) fn sort_items(&mut self, ui: &mut Ui) {
        for key in SortKey::ALL {
            if ui.radio(self.browser.sort == Some(key), tr(key.name())).clicked() {
                self.act(Action::SortBy(key));
                ui.close();
            }
        }
        ui.separator();
        let mut asc = !self.browser.descending;
        if ui.checkbox(&mut asc, tr(S::Ascending)).clicked() {
            self.act(Action::ToggleDescending);
        }
    }

    pub(crate) fn app_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if icons::button(ui, &t, Icon::Back, BAR_ICON, tr(S::TipBack), !self.browser.back.is_empty()).clicked() {
                self.act(Action::Back);
            }
            if icons::button(ui, &t, Icon::Forward, BAR_ICON, tr(S::TipForward), !self.browser.forward.is_empty()).clicked() {
                self.act(Action::Forward);
            }
            icons::menu_button(ui, &t, Icon::Folder, BAR_ICON, tr(S::TipParent), |ui| {
                let ancestors: Vec<_> = self.browser.path.ancestors().skip(1).map(|p| p.to_path_buf()).collect();
                for p in ancestors {
                    self.mi_text(ui, &pier_engine::fs::display_name(&p), Action::GoTo(p.clone()));
                }
                ui.separator();
                for (place, p) in pier_engine::fs::default_places() {
                    self.mi_text(ui, tr(crate::panels::place_name(place)), Action::GoTo(p));
                }
            });
            icons::menu_button(ui, &t, Icon::Clock, BAR_ICON, tr(S::TipRecent), |ui| {
                ui.label(RichText::new(tr(S::RecentFolders)).color(t.fg_dim).small());
                let recent: Vec<_> = self.browser.recent.iter().skip(1).cloned().collect();
                if recent.is_empty() {
                    ui.label(RichText::new(tr(S::NoRecent)).color(t.fg_disabled));
                }
                for p in recent {
                    self.mi_text(ui, &pier_engine::fs::display_name(&p), Action::GoTo(p.clone()));
                }
            });
            ui.add_space(8.0);
            if icons::button(ui, &t, Icon::Camera, BAR_ICON, tr(S::TipCamera), true).clicked() {
                self.soon(S::GetPhotosFromCamera);
            }
            icons::menu_button(ui, &t, Icon::Refine, BAR_ICON, tr(S::TipRefine), |ui| {
                self.mi_soon(ui, S::ReviewMode);
                self.mi_soon(ui, S::BatchRename);
                self.mi_soon(ui, S::FileInfo);
            });
            if icons::button(ui, &t, Icon::Aperture, BAR_ICON, tr(S::TipCameraRaw), !self.browser.selection.is_empty()).clicked() {
                self.act(Action::OpenInCameraRaw);
            }
            ui.add_space(8.0);
            if icons::button(ui, &t, Icon::RotateCcw, BAR_ICON, tr(S::TipRotateCcw), true).clicked() {
                self.soon(S::RotateCcw);
            }
            if icons::button(ui, &t, Icon::RotateCw, BAR_ICON, tr(S::TipRotateCw), true).clicked() {
                self.soon(S::RotateCw);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(4.0);
                let field = egui::TextEdit::singleline(&mut self.browser.search).hint_text(tr(S::QuickSearch)).desired_width(190.0);
                if ui.add(field).changed() {
                    self.browser.refresh_visible();
                }
                let (r, _) = ui.allocate_exact_size(Vec2::splat(SMALL_ICON), Sense::hover());
                icons::paint(ui.painter(), r.shrink(3.0), Icon::Search, t.fg_dim);
                ui.add_space(14.0);
                icons::menu_button(ui, &t, Icon::Caret, 14.0, tr(S::Workspace), |ui| {
                    for ws in Workspace::ALL {
                        if ui.add(Button::new(tr(ws.name())).selected(self.ui.workspace == ws)).clicked() {
                            self.act(Action::Workspace(ws));
                        }
                    }
                    ui.separator();
                    self.mi_soon(ui, S::NewWorkspace);
                    self.mi(ui, S::ResetWorkspace, None, Action::ResetWorkspace);
                });
                let mut shown: Vec<Workspace> = Workspace::ALL.into_iter().take(Workspace::IN_BAR).collect();
                // The active workspace is always in the bar: it takes the last slot when it would be in the ▾ menu.
                if !shown.contains(&self.ui.workspace)
                    && let Some(last) = shown.last_mut()
                {
                    *last = self.ui.workspace;
                }
                for (i, ws) in shown.into_iter().enumerate().rev() {
                    self.workspace_tab(ui, ws);
                    if i > 0 {
                        ui.label(RichText::new("|").color(t.separator.gamma_multiply(2.0)));
                    }
                }
            });
        });
    }

    fn workspace_tab(&mut self, ui: &mut Ui, ws: Workspace) {
        let t = &self.tokens;
        let active = self.ui.workspace == ws;
        let text = RichText::new(tr(ws.name())).color(if active { t.accent } else { t.fg_dim });
        let resp = ui.add(egui::Label::new(text).sense(Sense::click()));
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, active, tr(ws.name())));
        if active {
            let r = resp.rect;
            ui.painter().line_segment([egui::pos2(r.left(), r.bottom() + 3.0), egui::pos2(r.right(), r.bottom() + 3.0)], egui::Stroke::new(2.0, t.accent));
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.act(Action::Workspace(ws));
        }
    }

    pub(crate) fn path_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            let crumbs = pier_engine::fs::breadcrumbs(&self.browser.path);
            let right_width = 330.0;
            let crumb_width = (ui.available_width() - right_width).max(80.0);
            ui.allocate_ui(vec2(crumb_width, ui.available_height()), |ui| {
                ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                ui.horizontal_centered(|ui| {
                    for (i, crumb) in crumbs.iter().enumerate() {
                        let name = if i == 0 { tr(S::Computer).to_owned() } else { pier_engine::fs::display_name(crumb) };
                        let last = i + 1 == crumbs.len();
                        let text = RichText::new(name).color(if last { t.fg_strong } else { t.fg_text });
                        let resp = ui.add(egui::Label::new(text).sense(Sense::click()));
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, true, crumb.display().to_string()));
                        if resp.hovered() {
                            ui.painter().line_segment([resp.rect.left_bottom(), resp.rect.right_bottom()], egui::Stroke::new(1.0, t.fg_dim));
                        }
                        if resp.clicked() {
                            self.act(Action::GoTo(crumb.clone()));
                        }
                        self.want_subfolders(crumb);
                        let subs = self.subfolders.get(crumb).cloned().flatten().and_then(Result::ok).unwrap_or_default();
                        icons::popup_button(ui, &t, Icon::TriangleRight, 14.0, tr(S::TipSubfolders), false, |ui| {
                            egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                                for s in &subs {
                                    self.mi_text(ui, &pier_engine::fs::display_name(s), Action::GoTo(s.clone()));
                                }
                            });
                        });
                    }
                });
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                if icons::button(ui, &t, Icon::Trash, SMALL_ICON, tr(S::TipDelete), true).clicked() {
                    self.soon(S::MoveToTrash);
                }
                if icons::button(ui, &t, Icon::NewFolder, SMALL_ICON, tr(S::TipNewFolder), true).clicked() {
                    self.soon(S::NewFolder);
                }
                ui.add_space(6.0);
                let icon = if self.browser.descending { Icon::SortDesc } else { Icon::SortAsc };
                if icons::button(ui, &t, icon, SMALL_ICON, tr(S::TipSortOrder), true).clicked() {
                    self.browser.descending = !self.browser.descending;
                    self.browser.refresh_visible();
                }
                let key = self.browser.sort.unwrap_or(SortKey::Name);
                let sort_text = tr(S::SortBy).replace("{key}", tr(key.name()));
                ui.menu_button(RichText::new(format!("{sort_text} ▼")).color(t.fg_text), |ui| self.sort_items(ui));
                ui.add_space(6.0);
                icons::menu_button(ui, &t, Icon::Star, SMALL_ICON, tr(S::TipFilterRating), |ui| self.quick_filter_items(ui));
                icons::menu_button(ui, &t, Icon::Picture, SMALL_ICON, tr(S::TipQuality), |ui| {
                    for (i, s) in [S::QualityEmbedded, S::QualityOnDemand, S::QualityAlways].into_iter().enumerate() {
                        let text = format!("{}   ({})", tr(s), tr(S::ComingSoon));
                        if ui.radio(i == 0, text).clicked() {
                            self.soon(s);
                        }
                    }
                    ui.separator();
                    self.mi_soon(ui, S::Quality100);
                });
            });
        });
    }

    fn quick_filter_items(&mut self, ui: &mut Ui) {
        let mut choices: Vec<(String, QuickFilter)> =
            vec![(tr(S::ShowAllItems).to_owned(), QuickFilter::All), (tr(S::ShowUnrated).to_owned(), QuickFilter::Unrated)];
        for n in 1..=5 {
            choices.push((tr(S::ShowRatedAtLeast).replace("{n}", &"★".repeat(n as usize)), QuickFilter::AtLeast(n)));
        }
        choices.push((tr(S::ShowLabeled).to_owned(), QuickFilter::Labeled));
        choices.push((tr(S::ShowUnlabeled).to_owned(), QuickFilter::Unlabeled));
        choices.push((tr(S::ShowRejectedOnly).to_owned(), QuickFilter::RejectedOnly));
        for (text, q) in choices {
            if ui.radio(self.browser.quick == q, text).clicked() {
                self.browser.quick = q;
                self.browser.refresh_visible();
                ui.close();
            }
        }
    }

    pub(crate) fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.horizontal_centered(|ui| {
            let n = self.browser.visible.len();
            let m = self.browser.selection.len();
            let text = if self.browser.loading {
                tr(S::Loading).to_owned()
            } else if m == 0 {
                tr(S::ItemsStatus).replace("{n}", &n.to_string())
            } else {
                tr(S::ItemsSelected)
                    .replace("{n}", &n.to_string())
                    .replace("{m}", &m.to_string())
                    .replace("{size}", &crate::shell::fmt_size(self.browser.selected_size()))
            };
            ui.label(RichText::new(text).color(t.fg_dim));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let view = self.ui.view();
                for (icon, tip, mode) in [
                    (Icon::ViewList, S::TipViewList, ViewMode::List),
                    (Icon::ViewDetails, S::TipViewDetails, ViewMode::Details),
                    (Icon::ViewThumbs, S::TipViewThumbs, ViewMode::Thumbnails),
                ] {
                    let resp = icons::button(ui, &t, icon, SMALL_ICON, tr(tip), true);
                    if view == mode {
                        ui.painter().rect_stroke(resp.rect, t.radius_small, egui::Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
                    }
                    if resp.clicked() {
                        self.act(Action::View(mode));
                    }
                }
                if icons::button(ui, &t, Icon::Lock, SMALL_ICON, tr(S::TipGridLock), true).clicked() {
                    self.soon(S::GridLock);
                }
                ui.add_space(8.0);
                if icons::button(ui, &t, Icon::LargeSquare, SMALL_ICON, tr(S::TipLarger), true).clicked() {
                    self.act(Action::ThumbStep(32.0));
                }
                ui.spacing_mut().slider_width = 130.0;
                let slider = ui.add(egui::Slider::new(&mut self.ui.thumb_size, THUMB_MIN..=THUMB_MAX).show_value(false));
                slider.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Slider, true, tr(S::TipLarger)));
                if icons::button(ui, &t, Icon::SmallSquare, SMALL_ICON, tr(S::TipSmaller), true).clicked() {
                    self.act(Action::ThumbStep(-32.0));
                }
            });
        });
    }
}
