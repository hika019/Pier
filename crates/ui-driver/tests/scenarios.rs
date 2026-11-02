//! Scenario tests: the real shell, driven offscreen with real mouse and keyboard input.

use egui::{Key, Modifiers, PointerButton, pos2};
use std::path::PathBuf;

use pier_engine::craft::Launch;
use pier_ui_driver::{Config, Driver};
use pier_ui_egui::i18n::Lang;
use pier_ui_egui::state::{Dialog, PrefPage};
use pier_ui_egui::theme::Brightness;
use pier_ui_egui::workspace::Workspace;

fn driver(workspace: Workspace) -> Driver {
    Driver::new(Config { workspace, ..Config::default() }).unwrap()
}

fn cleanup(d: Driver) {
    let dir = d.dir.clone();
    drop(d);
    let _ = std::fs::remove_dir_all(dir);
}

/// Regression: every frame (so every mouse move, in the app) shrank the top panel of each column by
/// one point, because a panel group claimed one point less than it was given.
#[test]
fn panels_stay_put_while_the_mouse_moves() {
    for ws in Workspace::ALL {
        let mut d = driver(ws);
        let before = d.layout();
        d.move_path(pos2(5.0, 450.0), pos2(1435.0, 450.0), 60);
        d.move_path(pos2(720.0, 100.0), pos2(720.0, 880.0), 30);
        assert_eq!(d.layout(), before, "{ws:?}: panel groups moved while only the mouse moved");
        cleanup(d);
    }
}

#[test]
fn clicking_a_workspace_name_switches_workspace() {
    let mut d = driver(Workspace::Essentials);
    d.click_label("Filmstrip", PointerButton::Primary).unwrap();
    assert_eq!(d.state()["workspace"], "filmstrip");
    cleanup(d);
}

#[test]
fn click_selects_and_cmd_digit_rates() {
    let mut d = driver(Workspace::Essentials);
    d.click_label("aurora-03.jpg", PointerButton::Primary).unwrap();
    assert_eq!(d.state()["selection"], serde_json::json!(["aurora-03.jpg"]));
    d.key(Key::Num4, Modifiers::COMMAND);
    let path = d.dir.join("aurora-03.jpg");
    assert_eq!(d.shell().browser().rating(&path), 4);
    cleanup(d);
}

#[test]
fn double_clicking_a_folder_opens_it_and_back_returns() {
    let mut d = driver(Workspace::Essentials);
    let w = d.find("Selects", false).unwrap();
    d.double_click_at(w.rect.center());
    d.settle();
    assert!(d.state()["folder"].as_str().unwrap().ends_with("Selects"));
    d.click_label("Go back", PointerButton::Primary).unwrap();
    d.settle();
    assert_eq!(d.state()["folder"], d.dir.display().to_string());
    cleanup(d);
}

#[test]
fn space_opens_full_screen_and_escape_closes_it() {
    let mut d = driver(Workspace::Essentials);
    d.click_label("dunes-01.jpg", PointerButton::Primary).unwrap();
    d.key(Key::Space, Modifiers::NONE);
    assert_eq!(d.state()["fullScreen"], "dunes-01.jpg");
    d.key(Key::Escape, Modifiers::NONE);
    assert!(d.state()["fullScreen"].is_null());
    cleanup(d);
}

#[test]
fn japanese_interface_is_clickable_by_label() {
    let mut d = Driver::new(Config { lang: Lang::Ja, ..Config::default() }).unwrap();
    d.click_label("フィルムストリップ", PointerButton::Primary).unwrap();
    assert_eq!(d.state()["workspace"], "filmstrip");
    cleanup(d);
}

#[test]
fn menus_run_commands_and_say_when_something_is_coming_soon() {
    let mut d = driver(Workspace::Essentials);
    d.click_label("View", PointerButton::Primary).unwrap();
    d.click_label("As List", PointerButton::Primary).unwrap();
    assert_eq!(d.state()["view"], "List");
    d.click_label("Tools", PointerButton::Primary).unwrap();
    d.click_label("Batch Rename… (coming soon)", PointerButton::Primary).unwrap();
    assert_eq!(d.state()["toast"], "Coming soon: Batch Rename");
    cleanup(d);
}

fn dialog_rect(d: &Driver) -> serde_json::Value {
    d.state()["dialogRect"].clone()
}

/// Regression: a dialog grew by 17 points every frame (so every mouse move, in the app) until its
/// buttons left the screen.
#[test]
fn dialogs_keep_their_size_while_the_mouse_moves() {
    let mut dialogs: Vec<Dialog> = PrefPage::ALL.into_iter().map(Dialog::Preferences).collect();
    dialogs.extend([Dialog::About, Dialog::CraftApps, Dialog::CraftMissing { app: pier_engine::craft::PHOTOCRAFT, files: vec!["/tmp/a.psd".into()] }]);
    let mut d = driver(Workspace::Essentials);
    for dialog in dialogs {
        d.shell_mut().ui_state_mut().dialog = Some(dialog.clone());
        d.frames(3);
        let before = dialog_rect(&d);
        assert!(before.is_array(), "{dialog:?}: no dialog drawn");
        d.move_path(pos2(100.0, 100.0), pos2(1300.0, 800.0), 30);
        assert_eq!(dialog_rect(&d), before, "{dialog:?}: the dialog changed size while only the mouse moved");
    }
    cleanup(d);
}

#[test]
fn preferences_pages_are_picked_by_clicking_their_names() {
    let mut d = driver(Workspace::Essentials);
    d.key(Key::K, Modifiers::COMMAND);
    for page in ["Advanced", "Interface", "File Type Associations", "Output", "General"] {
        d.click_label(page, PointerButton::Primary).unwrap();
        assert_eq!(d.state()["dialog"], format!("Preferences({})", page_variant(page)), "clicked {page}");
    }
    cleanup(d);
}

fn page_variant(name: &str) -> &'static str {
    match name {
        "Advanced" => "Advanced",
        "Interface" => "Interface",
        "File Type Associations" => "FileTypes",
        "Output" => "Output",
        _ => "General",
    }
}

/// Regression: every dialog shared one window id, so About opened after Preferences kept its height.
#[test]
fn a_dialog_does_not_inherit_the_size_of_the_previous_one() {
    let mut d = driver(Workspace::Essentials);
    d.shell_mut().ui_state_mut().dialog = Some(Dialog::About);
    d.frames(3);
    let fresh = dialog_rect(&d);
    d.shell_mut().ui_state_mut().dialog = None;
    d.frames(2);
    d.shell_mut().ui_state_mut().dialog = Some(Dialog::Preferences(PrefPage::FileTypes));
    d.frames(3);
    d.shell_mut().ui_state_mut().dialog = Some(Dialog::About);
    d.frames(3);
    assert_eq!(dialog_rect(&d), fresh);
    cleanup(d);
}

/// Half a second at 60 Hz: longer than egui's double-click window (0.3 s), so two clicks stay two.
const DOUBLE_CLICK_GAP_FRAMES: usize = 30;

fn with_all_craft_apps() -> Driver {
    Driver::new(Config { installed: pier_engine::craft::ALL.to_vec(), ..Config::default() }).unwrap()
}

fn launched(d: &Driver) -> Vec<Launch> {
    d.launches.take()
}

fn app(name: &str, files: &[PathBuf]) -> Launch {
    Launch::App { app: PathBuf::from(format!("/Applications/{name}.app")), files: files.to_vec() }
}

/// Bridge: double-clicking a camera raw opens it in Photoshop, which shows Camera Raw first. Pier
/// hands it to PhotoCraft, which does the same (its open-time Camera Raw dialog).
#[test]
fn double_clicking_a_raw_opens_it_in_photocraft() {
    let mut d = with_all_craft_apps();
    let w = d.find("IMG_0412.CR3", false).unwrap();
    d.double_click_at(w.rect.center());
    assert_eq!(launched(&d), [app("PhotoCraft", &[d.dir.join("IMG_0412.CR3")])]);
    cleanup(d);
}

#[test]
fn double_clicking_a_jpeg_opens_it_in_photocraft() {
    let mut d = with_all_craft_apps();
    let w = d.find("dunes-01.jpg", false).unwrap();
    d.double_click_at(w.rect.center());
    assert_eq!(launched(&d), [app("PhotoCraft", &[d.dir.join("dunes-01.jpg")])]);
    cleanup(d);
}

/// File › Open, Return and Cmd+Down open like a double-click (Bridge help, "Open files"); one launch
/// per app, carrying all of that app's files.
#[test]
fn open_hands_each_app_its_files_in_one_launch() {
    let mut d = with_all_craft_apps();
    for (key, mods) in [(Key::O, Modifiers::COMMAND), (Key::Enter, Modifiers::NONE), (Key::ArrowDown, Modifiers::COMMAND)] {
        d.click_label("IMG_0412.CR3", PointerButton::Primary).unwrap();
        for name in ["poster.psd", "teaser.mp4"] {
            d.frames(DOUBLE_CLICK_GAP_FRAMES);
            d.click_at(d.find(name, false).unwrap().rect.center(), PointerButton::Primary, Modifiers::COMMAND);
        }
        assert_eq!(d.state()["selection"], serde_json::json!(["IMG_0412.CR3", "poster.psd", "teaser.mp4"]), "{key:?}");
        d.key(key, mods);
        d.frames(DOUBLE_CLICK_GAP_FRAMES);
        let mut got = launched(&d);
        got.sort_by_key(|l| format!("{l:?}"));
        let photo = app("PhotoCraft", &[d.dir.join("IMG_0412.CR3"), d.dir.join("poster.psd")]);
        let film = app("FilmCraft", &[d.dir.join("teaser.mp4")]);
        assert_eq!(got, [film, photo], "{key:?} {mods:?}");
    }
    cleanup(d);
}

/// Without PhotoCraft nothing is started: Pier says which app is needed and how to get it.
#[test]
fn opening_without_photocraft_asks_to_install_it() {
    let mut d = driver(Workspace::Essentials);
    let w = d.find("IMG_0412.CR3", false).unwrap();
    d.double_click_at(w.rect.center());
    assert!(launched(&d).is_empty());
    let dialog = d.state()["dialog"].as_str().unwrap_or_default().to_owned();
    assert!(dialog.starts_with("CraftMissing") && dialog.contains("photocraft") && dialog.contains("IMG_0412.CR3"), "{dialog}");
    cleanup(d);
}

/// The page list of Bridge 2026's Preferences on macOS: General and Advanced first, then A–Z
/// (Bridge help, "Set workflow preferences" screenshot; not yet checked on a running Bridge).
#[test]
fn preferences_list_their_pages_in_bridge_order() {
    let mut d = driver(Workspace::Essentials);
    d.key(Key::K, Modifiers::COMMAND);
    // The list's rows: everything in the dialog's column of rows that starts with General (the
    // content behind the dialog is in the accessibility tree too).
    let dialog = rect_of(&d.state()["dialogRect"]);
    let first = d.find("General", false).unwrap().rect;
    let in_list = |r: egui::Rect| dialog.contains_rect(r) && r.left() == first.left() && r.width() == first.width();
    let mut items: Vec<_> = d.widgets().into_iter().filter(|w| w.role == "ListItem" && in_list(w.rect)).collect();
    items.sort_by(|a, b| a.rect.top().total_cmp(&b.rect.top()));
    let names: Vec<&str> = items.iter().map(|w| w.label.as_str()).collect();
    let bridge = [
        "General",
        "Advanced",
        "Applications",
        "Cache",
        "Cache Management",
        "Content",
        "Export",
        "File Type Associations",
        "Interface",
        "Keywords",
        "Labels and Ratings",
        "Media Cache",
        "Metadata",
        "Output",
        "Playback",
        "Quick Actions",
        "Startup Scripts",
        "Thumbnails",
        "Workflow",
    ];
    assert_eq!(names, bridge);
    cleanup(d);
}

fn rect_of(v: &serde_json::Value) -> egui::Rect {
    let n = |i: usize| v[i].as_f64().unwrap() as f32;
    egui::Rect::from_min_size(pos2(n(0), n(1)), egui::vec2(n(2), n(3)))
}

/// Preferences end with Cancel and OK (Bridge help screenshot, macOS: Cancel left of OK). OK keeps
/// the changes; Cancel and Escape drop them.
#[test]
fn preferences_keep_changes_on_ok_and_drop_them_on_cancel_or_escape() {
    let mut d = driver(Workspace::Essentials);
    for (close, kept) in [("Cancel", false), ("Escape", false), ("OK", true)] {
        d.key(Key::K, Modifiers::COMMAND);
        d.click_label("Interface", PointerButton::Primary).unwrap();
        d.click_label("Lightest", PointerButton::Primary).unwrap();
        match close {
            "Escape" => d.key(Key::Escape, Modifiers::NONE),
            label => {
                d.click_label(label, PointerButton::Primary).unwrap();
            }
        }
        d.frames(2);
        assert!(d.state()["dialog"].is_null(), "{close}: still open");
        let brightness = d.shell().ui_state().brightness;
        assert_eq!(brightness == Brightness::Lightest, kept, "{close}: {brightness:?}");
    }
    cleanup(d);
}

#[test]
fn cancel_sits_left_of_ok_in_preferences() {
    let mut d = driver(Workspace::Essentials);
    d.key(Key::K, Modifiers::COMMAND);
    let cancel = d.find("Cancel", false).unwrap().rect;
    let ok = d.find("OK", false).unwrap().rect;
    assert!(cancel.right() <= ok.left() && (cancel.center().y - ok.center().y).abs() < 1.0, "Cancel {cancel:?} OK {ok:?}");
    cleanup(d);
}

/// The help's Japanese names for the pages it mentions.
#[test]
fn japanese_preferences_use_bridge_page_names() {
    let mut d = Driver::new(Config { lang: Lang::Ja, ..Config::default() }).unwrap();
    d.key(Key::K, Modifiers::COMMAND);
    for (page, variant) in [("詳細設定", "Advanced"), ("インターフェイス", "Interface"), ("ワークフロー", "Workflow")] {
        d.click_label(page, PointerButton::Primary).unwrap();
        assert_eq!(d.state()["dialog"], format!("Preferences({variant})"));
    }
    cleanup(d);
}

/// Cmd+R stays "Open in Camera Raw": LightCraft until dashboard.md #3 is decided.
#[test]
fn open_in_camera_raw_hands_the_raw_to_lightcraft() {
    let mut d = with_all_craft_apps();
    d.click_label("IMG_0412.CR3", PointerButton::Primary).unwrap();
    d.key(Key::R, Modifiers::COMMAND);
    assert_eq!(launched(&d), [app("LightCraft", &[d.dir.join("IMG_0412.CR3")])]);
    cleanup(d);
}

#[test]
fn cmd_click_adds_to_the_selection_and_shift_click_selects_a_range() {
    let mut d = driver(Workspace::Essentials);
    d.click_label("aurora-03.jpg", PointerButton::Primary).unwrap();
    d.frames(DOUBLE_CLICK_GAP_FRAMES);
    d.click_at(d.find("coast-12.jpg", false).unwrap().rect.center(), PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(d.state()["selection"], serde_json::json!(["aurora-03.jpg", "coast-12.jpg"]));
    d.frames(DOUBLE_CLICK_GAP_FRAMES);
    d.click_at(d.find("canyon-10.jpg", false).unwrap().rect.center(), PointerButton::Primary, Modifiers::SHIFT);
    // Range from the anchor (coast-12, the last click) back to canyon-10, in the view's order.
    assert_eq!(d.state()["selection"], serde_json::json!(["canyon-10.jpg", "coast-12.jpg"]));
    cleanup(d);
}
