//! Scenario tests: the real shell, driven offscreen with real mouse and keyboard input.

use egui::{Key, Modifiers, PointerButton, pos2};
use pier_ui_driver::{Config, Driver};
use pier_ui_egui::i18n::Lang;
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
