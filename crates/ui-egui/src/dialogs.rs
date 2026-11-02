//! Modal dialogs and the full-screen preview (docs/ui-design.md §6–§7, docs/craft-integration.md §4).

use egui::{Align2, Color32, Context, Id, Modal, Rect, RichText, Sense, Ui, pos2, vec2};
use pier_engine::craft::{self, CraftApp};
use pier_engine::{AssetKind, ImageSize};

use crate::content::fit;
use crate::i18n::{Lang, LangSetting, S, tr};
use crate::shell::{PierShell, kind_name};
use crate::state::{Dialog, PrefPage};
use crate::theme::Brightness;

/// Preferences: page list width, page width and body height. One size for every page, so switching
/// pages never moves the buttons.
const PREF_LIST_WIDTH: f32 = 170.0;
const PREF_PAGE_WIDTH: f32 = 470.0;
const PREF_BODY_HEIGHT: f32 = 380.0;
/// Room for the separator between the page list and the page.
const PREF_SEPARATOR: f32 = 6.0;

/// A row of buttons, right-aligned, exactly one button high. A layout given the dialog's whole
/// remaining height would claim it, and a modal remembers the size its contents claimed.
fn button_row(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let size = vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), add);
}

impl PierShell {
    pub(crate) fn dialogs_ui(&mut self, ctx: &Context) {
        let Some(dialog) = self.ui.dialog.clone() else { return };
        let t = self.tokens.clone();
        let frame = egui::Frame::NONE.fill(t.bg_panel).stroke(t.separator_stroke()).corner_radius(6).inner_margin(16);
        let resp = Modal::new(Id::unique(("pier-dialog", std::mem::discriminant(&dialog)))).frame(frame).show(ctx, |ui| match dialog {
            Dialog::Preferences(page) => self.preferences(ui, page),
            Dialog::CraftMissing { app, files } => self.craft_missing(ui, app, &files),
            Dialog::CraftApps => self.craft_apps_dialog(ui),
            Dialog::About => self.about(ui),
        });
        self.probe.dialog = Some(resp.response.rect);
        if resp.should_close() {
            self.ui.dialog = None;
        }
    }

    fn preferences(&mut self, ui: &mut Ui, page: PrefPage) {
        let t = self.tokens.clone();
        ui.heading(tr(S::Preferences).trim_end_matches('…'));
        ui.add_space(8.0);
        let body = vec2(PREF_LIST_WIDTH + PREF_PAGE_WIDTH + PREF_SEPARATOR + ui.spacing().item_spacing.x * 2.0, PREF_BODY_HEIGHT);
        // The vertical separator fills the height it is offered, so offer it exactly the body's.
        ui.allocate_ui_with_layout(body, egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.set_min_size(body);
            ui.vertical(|ui| {
                ui.set_width(PREF_LIST_WIDTH);
                for p in PrefPage::ALL {
                    if ui.add_sized([PREF_LIST_WIDTH, 20.0], egui::Button::new(tr(p.name())).selected(p == page).frame(p == page)).clicked() {
                        self.ui.dialog = Some(Dialog::Preferences(p));
                    }
                }
            });
            ui.add(egui::Separator::default().spacing(PREF_SEPARATOR));
            ui.vertical(|ui| {
                ui.set_width(PREF_PAGE_WIDTH);
                ui.label(RichText::new(tr(page.name())).strong().color(t.fg_strong));
                ui.add_space(6.0);
                match page {
                    PrefPage::Interface => {
                        ui.label(tr(S::PrefColorTheme));
                        ui.horizontal(|ui| {
                            for b in Brightness::ALL {
                                let swatch = crate::theme::Tokens::for_brightness(b).bg_panel;
                                let (r, resp) = ui.allocate_exact_size(vec2(44.0, 28.0), Sense::click());
                                ui.painter().rect_filled(r, 3, swatch);
                                if self.ui.brightness == b {
                                    ui.painter().rect_stroke(r, 3, egui::Stroke::new(2.0, t.accent), egui::StrokeKind::Outside);
                                }
                                if resp.on_hover_text(b.label()).clicked() {
                                    self.ui.brightness = b;
                                }
                            }
                        });
                        ui.add_space(8.0);
                        ui.label(RichText::new(tr(S::PrefSoon)).color(t.fg_disabled).small());
                    }
                    PrefPage::Advanced => {
                        ui.label(tr(S::PrefLanguage));
                        for (setting, text) in
                            [(LangSetting::Auto, tr(S::PrefLangAuto)), (LangSetting::Fixed(Lang::En), "English"), (LangSetting::Fixed(Lang::Ja), "日本語")]
                        {
                            if ui.radio(self.ui.lang == setting, text).clicked() {
                                self.ui.lang = setting;
                            }
                        }
                        if self.cjk_font.is_none() {
                            ui.label(RichText::new("No Japanese system font was found; Japanese text may not draw.").color(t.reject).small());
                        }
                        ui.add_space(8.0);
                        ui.label(RichText::new(tr(S::PrefSoon)).color(t.fg_disabled).small());
                    }
                    PrefPage::FileTypes => {
                        egui::Grid::new("assoc").num_columns(3).spacing([16.0, 4.0]).striped(true).show(ui, |ui| {
                            for kind in AssetKind::FILTERABLE.into_iter().filter(|k| *k != AssetKind::Folder) {
                                ui.label(tr(kind_name(kind)));
                                match craft::app_for(kind) {
                                    Some(app) => {
                                        ui.label(app.name);
                                        let ok = self.craft_apps.get(app.id).is_some_and(Option::is_some);
                                        ui.label(RichText::new(tr(if ok { S::Installed } else { S::NotInstalled })).color(if ok {
                                            t.labels.approved
                                        } else {
                                            t.fg_dim
                                        }));
                                    }
                                    None => {
                                        ui.label(tr(S::DefaultApp));
                                        ui.label("");
                                    }
                                }
                                ui.end_row();
                            }
                        });
                        ui.add_space(8.0);
                        ui.label(RichText::new(tr(S::PrefFileTypesNote)).color(t.fg_disabled).small());
                    }
                    _ => {
                        ui.label(RichText::new(tr(S::PrefSoon)).color(t.fg_dim));
                    }
                }
            });
        });
        ui.separator();
        button_row(ui, |ui| {
            if ui.button(tr(S::Ok)).clicked() {
                ui.close();
            }
        });
    }

    fn craft_missing(&mut self, ui: &mut Ui, app: CraftApp, files: &[std::path::PathBuf]) {
        let t = self.tokens.clone();
        ui.set_width(520.0);
        ui.heading(tr(S::CraftNeededTitle).replace("{app}", app.name));
        ui.add_space(6.0);
        ui.label(tr(S::CraftNeededBody).replace("{app}", app.name));
        if let Some(f) = files.first() {
            ui.add_space(4.0);
            ui.label(RichText::new(f.display().to_string()).color(t.fg_dim).small());
        }
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            self.launcher_button(ui);
            if ui.button(tr(S::OpenAppPage).replace("{app}", app.name)).clicked() {
                let _ = craft::open_default(std::ffi::OsStr::new(&app.page_url()));
            }
            if ui.button(tr(S::OpenWithDefault)).clicked() {
                for f in files {
                    let _ = craft::open_default(f.as_os_str());
                }
                ui.close();
            }
            if ui.button(tr(S::Cancel)).clicked() {
                ui.close();
            }
        });
    }

    fn launcher_button(&mut self, ui: &mut Ui) {
        match self.launcher.clone() {
            Some(path) => {
                if ui.button(tr(S::OpenLauncher)).clicked() {
                    let _ = craft::open_in_app(&path, &[]);
                }
            }
            None => {
                if ui.button(tr(S::DownloadLauncher)).clicked() {
                    let _ = craft::open_default(std::ffi::OsStr::new(craft::LAUNCHER_DOWNLOAD_URL));
                }
            }
        }
    }

    fn craft_apps_dialog(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.set_width(520.0);
        ui.heading(tr(S::CraftAppsTitle));
        ui.add_space(4.0);
        ui.label(tr(S::CraftAppsBody));
        ui.add_space(8.0);
        egui::Grid::new("craft-apps").num_columns(3).spacing([18.0, 6.0]).striped(true).show(ui, |ui| {
            for app in craft::ALL {
                ui.label(RichText::new(app.name).strong());
                let ok = self.craft_apps.get(app.id).is_some_and(Option::is_some);
                ui.label(RichText::new(tr(if ok { S::Installed } else { S::NotInstalled })).color(if ok { t.labels.approved } else { t.fg_dim }));
                if ui.small_button(tr(S::OpenAppPage).replace("{app}", app.name)).clicked() {
                    let _ = craft::open_default(std::ffi::OsStr::new(&app.page_url()));
                }
                ui.end_row();
            }
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            self.launcher_button(ui);
            button_row(ui, |ui| {
                if ui.button(tr(S::Close)).clicked() {
                    ui.close();
                }
            });
        });
    }

    fn about(&mut self, ui: &mut Ui) {
        let t = self.tokens.clone();
        ui.set_width(420.0);
        ui.heading("Pier");
        ui.label(RichText::new(format!("{} {}", env!("CARGO_PKG_VERSION"), "(early prototype)")).color(t.fg_dim));
        ui.add_space(8.0);
        ui.label(tr(S::AboutBody));
        ui.label("MIT License · © 2026 hika019");
        ui.add_space(8.0);
        ui.label(RichText::new(tr(S::AboutTrademark)).color(t.fg_dim).small());
        ui.add_space(8.0);
        button_row(ui, |ui| {
            if ui.button(tr(S::Ok)).clicked() {
                ui.close();
            }
        });
    }

    pub(crate) fn full_screen_ui(&mut self, ctx: &Context) {
        let Some(path) = self.ui.full_screen.clone() else { return };
        let screen = ctx.content_rect();
        let name = pier_engine::fs::display_name(&path);
        let has_image = pier_engine::AssetKind::from_extension(&path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()).has_thumbnail();
        let img = if has_image { self.image(&path, ImageSize::Preview).or_else(|| self.image(&path, ImageSize::Thumb)) } else { None };
        let font = self.tokens.font();
        egui::Area::new(Id::unique("pier-full-screen")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(rect, 0, Color32::BLACK);
            if let Some((tex, size)) = img {
                ui.painter().image(tex, fit(rect.shrink(16.0), size), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
            ui.painter().text(pos2(rect.center().x, rect.bottom() - 14.0), Align2::CENTER_BOTTOM, name, font, Color32::from_gray(140));
            if resp.clicked() {
                self.soon(S::FullScreenPreview);
            }
        });
    }
}
