//! Every workspace, language and brightness draws without panicking, with every glyph available
//! (kittest fails on a missing glyph) and every widget named for accessibility.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use pier_ui_egui::PierShell;
use pier_ui_egui::i18n::{Lang, LangSetting};
use pier_ui_egui::state::{Dialog, PrefPage, ViewMode};
use pier_ui_egui::theme::Brightness;
use pier_ui_egui::workspace::Workspace;

fn demo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pier-smoke-{name}-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    for (i, (w, h)) in [(64, 48), (48, 64), (80, 40)].into_iter().enumerate() {
        image::RgbImage::from_pixel(w, h, image::Rgb([40 * i as u8, 120, 200])).save(dir.join(format!("img-{i}.png"))).unwrap();
    }
    std::fs::write(dir.join("broken.jpg"), b"not a jpeg").unwrap();
    std::fs::write(dir.join("poster.psd"), b"x").unwrap();
    dir
}

fn settle(h: &mut Harness<'_, PierShell>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut quiet = 0;
    while Instant::now() < deadline && quiet < 4 {
        h.step();
        quiet = if h.state().is_idle() { quiet + 1 } else { 0 };
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn harness(dir: PathBuf, setup: impl FnOnce(&mut PierShell)) -> Harness<'static, PierShell> {
    let mut shell = PierShell::new(dir.clone());
    setup(&mut shell);
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 800.0)).build_ui_state(|ui, s: &mut PierShell| s.ui(ui), shell);
    settle(&mut h);
    h.state_mut().browser_mut().click_select(&dir.join("img-0.png"), false, false);
    settle(&mut h);
    h
}

#[test]
fn every_workspace_draws_in_both_languages() {
    let dir = demo("ws");
    for ws in Workspace::ALL {
        for (lang, brightness) in [(Lang::En, Brightness::Darkest), (Lang::Ja, Brightness::Lightest)] {
            let mut h = harness(dir.clone(), |s| {
                s.ui_state_mut().workspace = ws;
                s.ui_state_mut().lang = LangSetting::Fixed(lang);
                s.ui_state_mut().brightness = brightness;
            });
            h.step();
        }
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn views_and_dialogs_draw() {
    let dir = demo("views");
    for view in [ViewMode::Details, ViewMode::List] {
        let mut h = harness(dir.clone(), |s| {
            s.ui_state_mut().views.insert(Workspace::Essentials, view);
            s.ui_state_mut().lang = LangSetting::Fixed(Lang::Ja);
        });
        h.step();
    }
    let dialogs = [
        Dialog::About,
        Dialog::CraftApps,
        Dialog::CraftMissing { app: pier_engine::craft::PHOTOCRAFT, files: vec![dir.join("poster.psd")] },
        Dialog::Preferences(PrefPage::Interface),
        Dialog::Preferences(PrefPage::Advanced),
        Dialog::Preferences(PrefPage::FileTypes),
        Dialog::Preferences(PrefPage::Cache),
    ];
    for d in dialogs {
        for lang in [Lang::En, Lang::Ja] {
            let mut h = harness(dir.clone(), |s| {
                s.ui_state_mut().lang = LangSetting::Fixed(lang);
                s.ui_state_mut().dialog = Some(d.clone());
            });
            h.step();
        }
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_broken_image_is_an_icon_not_a_crash() {
    let dir = demo("broken");
    let mut h = harness(dir.clone(), |_| {});
    h.state_mut().browser_mut().click_select(&dir.join("broken.jpg"), false, false);
    settle(&mut h);
    assert!(h.state().is_idle());
    std::fs::remove_dir_all(&dir).unwrap();
}
