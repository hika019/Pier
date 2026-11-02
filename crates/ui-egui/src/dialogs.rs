//! Modal dialogs and the full-screen preview (docs/ui-design.md §6–§7, docs/craft-integration.md §4).

use egui::{Align2, Color32, Context, Id, Modal, Rect, RichText, Sense, Ui, pos2, vec2};
use pier_engine::craft::{self, CraftApp, Launch};
use pier_engine::{AssetKind, ImageSize};

use crate::content::fit;
use crate::i18n::{Lang, LangSetting, S, tr};
use crate::shell::{PierShell, kind_name};
use crate::state::{Dialog, PrefDraft, PrefPage};
use crate::theme::Brightness;

/// Preferences: the page list's outer width and the page's width. One size for every page, so
/// switching pages never moves the buttons.
const PREF_LIST_WIDTH: f32 = 170.0;
const PREF_PAGE_WIDTH: f32 = 470.0;
/// Inset of the page names inside the list's frame.
const PREF_LIST_INSET: f32 = 4.0;
/// Room for the separator between the page list and the page.
const PREF_SEPARATOR: f32 = 6.0;
/// A page name's distance from the left edge of its row.
const PREF_ROW_TEXT_INSET: f32 = 6.0;

/// The button Return presses, drawn in the accent colour (Bridge's OK, per its help screenshots).
fn default_button(ui: &mut Ui, t: &crate::theme::Tokens, text: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(text).color(t.fg_on_accent)).fill(t.accent))
}

/// Return, unless a text field has it.
fn return_pressed(ui: &Ui) -> bool {
    ui.memory(|m| m.focused().is_none()) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
}

/// A row of buttons, right-aligned, exactly one button high. A layout given the dialog's whole
/// remaining height would claim it, and a modal remembers the size its contents claimed.
fn button_row(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let size = vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), add);
}

impl PierShell {
    pub(crate) fn dialogs_ui(&mut self, ctx: &Context) {
        // The draft lives exactly as long as Preferences is the open dialog, and the queue of missing
        // apps as long as their prompts are, however the dialog was replaced.
        if !matches!(self.ui.dialog, Some(Dialog::Preferences(_))) {
            self.ui.pref_draft = None;
        }
        if !matches!(self.ui.dialog, Some(Dialog::CraftMissing { .. })) {
            self.ui.craft_missing_queue.clear();
        }
        let Some(dialog) = self.ui.dialog.clone() else { return };
        let t = self.tokens.clone();
        let frame = egui::Frame::NONE.fill(t.bg_panel).stroke(t.separator_stroke()).corner_radius(6).inner_margin(16);
        let resp = Modal::new(Id::unique(("pier-dialog", std::mem::discriminant(&dialog)))).frame(frame).show(ctx, |ui| match &dialog {
            Dialog::Preferences(page) => self.preferences(ui, *page),
            Dialog::CraftMissing { app, files } => self.craft_missing(ui, *app, files),
            Dialog::CraftApps => self.craft_apps_dialog(ui),
            Dialog::About => self.about(ui),
        });
        self.probe.dialog = Some(resp.response.rect);
        if resp.should_close() {
            self.ui.pref_draft = None;
            // The next app still missing for the files being opened (see `open_selection`).
            self.ui.dialog = if matches!(dialog, Dialog::CraftMissing { .. }) { self.ui.craft_missing_queue.pop_front() } else { None };
        }
    }

    fn preferences(&mut self, ui: &mut Ui, page: PrefPage) {
        let t = self.tokens.clone();
        let mut draft = self.ui.pref_draft.unwrap_or_else(|| PrefDraft::of(&self.ui));
        ui.heading(tr(S::Preferences).trim_end_matches('…'));
        ui.add_space(8.0);
        // The list's frame holds every page name, one row each, inside its inset and stroke.
        let list_edge = PREF_LIST_INSET + t.separator_stroke().width;
        let body = vec2(
            PREF_LIST_WIDTH + PREF_PAGE_WIDTH + PREF_SEPARATOR + ui.spacing().item_spacing.x * 2.0,
            PrefPage::ALL.len() as f32 * crate::panels::ROW + 2.0 * list_edge,
        );
        // The vertical separator fills the height it is offered, so offer it exactly the body's.
        let used = ui.allocate_ui_with_layout(body, egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.set_min_size(body);
            let list = egui::Frame::NONE.fill(t.bg_content).stroke(t.separator_stroke()).inner_margin(PREF_LIST_INSET);
            list.show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(PREF_LIST_WIDTH - 2.0 * list_edge);
                    self.pref_page_list(ui, page);
                });
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
                                let chosen = draft.brightness == b;
                                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, chosen, tr(b.name())));
                                ui.painter().rect_filled(r, 3, swatch);
                                ui.painter().rect_stroke(r, 3, t.separator_stroke(), egui::StrokeKind::Inside);
                                if chosen {
                                    ui.painter().rect_stroke(r, 3, egui::Stroke::new(2.0, t.accent), egui::StrokeKind::Outside);
                                }
                                if resp.on_hover_text(tr(b.name())).clicked() {
                                    draft.brightness = b;
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
                            if ui.radio(draft.lang == setting, text).clicked() {
                                draft.lang = setting;
                            }
                        }
                        if self.cjk_font.is_none() {
                            ui.label(RichText::new(tr(S::NoJapaneseFont)).color(t.reject).small());
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
        self.probe.fixed.push(("preferences body", body, used.response.rect.size()));
        ui.separator();
        let (mut ok, mut cancel) = (return_pressed(ui), false);
        button_row(ui, |ui| {
            ok |= default_button(ui, &t, tr(S::Ok)).clicked();
            cancel = ui.button(tr(S::Cancel)).clicked();
        });
        if ok {
            self.ui.brightness = draft.brightness;
            self.ui.lang = draft.lang;
        } else {
            self.ui.pref_draft = Some(draft);
        }
        if ok || cancel {
            ui.close();
        }
    }

    fn pref_page_list(&mut self, ui: &mut Ui, page: PrefPage) {
        let t = self.tokens.clone();
        ui.spacing_mut().item_spacing.y = 0.0;
        for p in PrefPage::ALL {
            let name = tr(p.name());
            let (rect, resp) = crate::panels::row(ui, &t, p == page, ("pref-page", p as u8), name);
            let color = if p == page { t.fg_strong } else { t.fg_text };
            ui.painter().text(pos2(rect.left() + PREF_ROW_TEXT_INSET, rect.center().y), Align2::LEFT_CENTER, name, t.font(), color);
            if resp.clicked() {
                self.ui.dialog = Some(Dialog::Preferences(p));
            }
        }
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
                let _ = self.launch(Launch::Default(app.page_url().into()));
            }
            if ui.button(tr(S::OpenWithDefault)).clicked() {
                for f in files {
                    let _ = self.launch(Launch::Default(f.clone().into_os_string()));
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
                    let _ = self.launch(Launch::App { app: path, files: Vec::new() });
                }
            }
            None => {
                if ui.button(tr(S::DownloadLauncher)).clicked() {
                    let _ = self.launch(Launch::Default(craft::LAUNCHER_DOWNLOAD_URL.into()));
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
                    let _ = self.launch(Launch::Default(app.page_url().into()));
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
        ui.label(RichText::new(format!("{} {}", env!("CARGO_PKG_VERSION"), tr(S::EarlyPrototype))).color(t.fg_dim));
        ui.add_space(8.0);
        ui.label(tr(S::AboutBody));
        ui.label("MIT License · © 2026 hika019");
        ui.add_space(8.0);
        ui.label(RichText::new(tr(S::AboutTrademark)).color(t.fg_dim).small());
        ui.add_space(8.0);
        let ok = return_pressed(ui);
        button_row(ui, |ui| {
            if default_button(ui, &t, tr(S::Ok)).clicked() || ok {
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    /// Regression: the button row claimed all the height the dialog offered, and a modal offers
    /// what it measured last frame, so a dialog never shrank once it had grown.
    #[test]
    fn a_dialog_shrinks_when_its_contents_do() {
        let mut tall = true;
        let mut harness = Harness::builder().with_size(vec2(800.0, 800.0)).build_ui_state(
            |ui, (tall, height): &mut (bool, f32)| {
                let resp = Modal::new(Id::unique("shrink-test")).show(ui.ctx(), |ui| {
                    ui.add_space(if *tall { 400.0 } else { 40.0 });
                    button_row(ui, |ui| {
                        let _ = ui.button("OK");
                    });
                });
                *height = resp.response.rect.height();
            },
            (tall, 0.0),
        );
        harness.run_steps(3);
        let grown = harness.state().1;
        tall = false;
        harness.state_mut().0 = tall;
        harness.run_steps(3);
        assert!(harness.state().1 < grown - 300.0, "still {} after shrinking from {grown}", harness.state().1);
    }
}
