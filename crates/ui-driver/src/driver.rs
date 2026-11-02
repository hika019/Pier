use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use pier_engine::craft::CraftApp;
use pier_ui_egui::PierShell;
use pier_ui_egui::i18n::{Lang, LangSetting};
use pier_ui_egui::theme::Brightness;
use pier_ui_egui::workspace::Workspace;
use serde_json::{Value, json};

/// Background work (listing, decoding) gets this long to finish in [`Driver::settle`].
const SETTLE_TIMEOUT: Duration = Duration::from_secs(10);
/// Simulated time per frame: a 60 Hz display, as the app runs (kittest's default of 0.25 s turns a
/// double-click into two clicks).
const FRAME_DT: f32 = 1.0 / 60.0;
/// Frames in a row with nothing pending before [`Driver::settle`] returns.
const QUIET_FRAMES: usize = 3;

#[derive(Clone, Debug)]
pub struct Config {
    /// The folder to open; `None` writes a fresh demo folder.
    pub dir: Option<PathBuf>,
    pub size: Vec2,
    pub scale: f32,
    pub workspace: Workspace,
    pub lang: Lang,
    pub theme: Brightness,
    /// The Craft Apps the fake desktop reports as installed. Nothing is ever started.
    pub installed: Vec<CraftApp>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir: None,
            size: egui::vec2(1440.0, 900.0),
            scale: 1.0,
            workspace: Workspace::Essentials,
            lang: Lang::En,
            theme: Brightness::Darkest,
            installed: Vec::new(),
        }
    }
}

/// A widget in the accessibility tree: what an agent can click by label.
#[derive(Clone, Debug, PartialEq)]
pub struct Widget {
    pub role: String,
    pub label: String,
    pub rect: Rect,
}

pub struct Driver {
    harness: Harness<'static, PierShell>,
    pub dir: PathBuf,
    /// What Pier asked the desktop to start, in order.
    pub launches: crate::desktop::Launches,
    pointer: Pos2,
    modifiers: Modifiers,
}

impl Driver {
    pub fn new(config: Config) -> Result<Self, String> {
        let dir = match config.dir {
            Some(d) => d,
            None => crate::demo::temp("driver")?,
        };
        let launches = crate::desktop::Launches::default();
        let mut shell = PierShell::new(dir.clone());
        shell.set_desktop(Box::new(crate::desktop::FakeDesktop::new(config.installed, launches.clone())));
        shell.ui_state_mut().workspace = config.workspace;
        shell.ui_state_mut().lang = LangSetting::Fixed(config.lang);
        shell.ui_state_mut().brightness = config.theme;
        let harness = Harness::builder()
            .with_size(config.size)
            .with_pixels_per_point(config.scale)
            .with_step_dt(FRAME_DT)
            .build_ui_state(|ui, s: &mut PierShell| s.ui(ui), shell);
        let mut d = Self { harness, dir, launches, pointer: pos2(-10.0, -10.0), modifiers: Modifiers::NONE };
        d.settle();
        Ok(d)
    }

    pub fn shell(&self) -> &PierShell {
        self.harness.state()
    }

    pub fn shell_mut(&mut self) -> &mut PierShell {
        self.harness.state_mut()
    }

    /// Runs one frame.
    pub fn frame(&mut self) {
        self.harness.step();
    }

    pub fn frames(&mut self, n: usize) {
        for _ in 0..n {
            self.harness.step();
        }
    }

    /// Steps frames until no listing, decode or metadata read is pending. Returns false on timeout.
    pub fn settle(&mut self) -> bool {
        let deadline = Instant::now() + SETTLE_TIMEOUT;
        let mut quiet = 0;
        while Instant::now() < deadline {
            self.harness.step();
            quiet = if self.shell().is_idle() { quiet + 1 } else { 0 };
            if quiet >= QUIET_FRAMES {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }

    pub fn resize(&mut self, size: Vec2) {
        self.harness.set_size(size);
        self.harness.step();
    }

    /// Moves the pointer to `to` and runs a frame, like a real mouse move.
    pub fn move_to(&mut self, to: Pos2) {
        self.pointer = to;
        self.harness.event(Event::PointerMoved(to));
        self.harness.step();
    }

    /// Moves the pointer from `from` to `to` in `steps` frames.
    pub fn move_path(&mut self, from: Pos2, to: Pos2, steps: usize) {
        let steps = steps.max(1);
        for i in 0..=steps {
            self.move_to(from.lerp(to, i as f32 / steps as f32));
        }
    }

    pub fn press(&mut self, button: PointerButton, pressed: bool) {
        self.harness.event(Event::PointerButton { pos: self.pointer, button, pressed, modifiers: self.modifiers });
        self.harness.step();
    }

    pub fn click_at(&mut self, at: Pos2, button: PointerButton, modifiers: Modifiers) {
        self.move_to(at);
        // The app reads held keys from the input state, which the OS updates with ModifiersChanged;
        // the modifiers on the button event alone are not seen.
        self.modifiers = modifiers;
        self.harness.event(Event::ModifiersChanged(modifiers));
        self.press(button, true);
        self.press(button, false);
        self.modifiers = Modifiers::NONE;
        self.harness.event(Event::ModifiersChanged(Modifiers::NONE));
        // The frame that handled the click changed state after drawing; the app repaints, and so do we.
        self.harness.step();
    }

    /// Two clicks a frame apart, as a real double-click arrives (egui needs separate frames to see two).
    pub fn double_click_at(&mut self, at: Pos2) {
        self.move_to(at);
        for _ in 0..2 {
            self.press(PointerButton::Primary, true);
            self.press(PointerButton::Primary, false);
        }
    }

    /// Presses at `from`, moves to `to` in `steps` frames, releases.
    pub fn drag(&mut self, from: Pos2, to: Pos2, steps: usize) {
        self.move_to(from);
        self.press(PointerButton::Primary, true);
        self.move_path(from, to, steps);
        self.press(PointerButton::Primary, false);
    }

    pub fn scroll_at(&mut self, at: Pos2, delta: Vec2) {
        self.move_to(at);
        self.harness.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        });
        self.harness.step();
    }

    pub fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.harness.key_press_modifiers(modifiers, key);
        self.harness.step();
        self.harness.step();
    }

    pub fn type_text(&mut self, text: &str) {
        self.harness.event(Event::Text(text.to_owned()));
        self.harness.step();
    }

    /// The first widget whose accessible label is exactly `label` (or contains it, when `contains`).
    pub fn find(&self, label: &str, contains: bool) -> Option<Widget> {
        self.widgets().into_iter().find(|w| if contains { w.label.contains(label) } else { w.label == label })
    }

    /// Clicks the centre of the widget labelled `label`.
    pub fn click_label(&mut self, label: &str, button: PointerButton) -> Result<Widget, String> {
        let w = self.find(label, false).or_else(|| self.find(label, true)).ok_or_else(|| format!("no widget labelled {label:?}"))?;
        self.click_at(w.rect.center(), button, Modifiers::NONE);
        Ok(w)
    }

    /// Every labelled widget in the accessibility tree.
    pub fn widgets(&self) -> Vec<Widget> {
        self.harness
            .query_all_by(|n| n.label().is_some_and(|l| !l.is_empty()))
            .map(|node| Widget {
                role: format!("{:?}", node.accesskit_node().role()),
                label: node.accesskit_node().label().unwrap_or_default(),
                rect: node.rect(),
            })
            .collect()
    }

    pub fn screenshot(&mut self, path: &Path) -> Result<(), String> {
        let img = self.harness.render()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        img.save(path).map_err(|e| e.to_string())
    }

    /// Each panel group's rect, by group id.
    pub fn layout(&self) -> Vec<(String, Rect)> {
        self.shell().probe().groups.iter().map(|g| (g.id.clone(), g.rect)).collect()
    }

    /// What the window shows, as JSON: workspace, folder, selection, dialog and every panel group.
    pub fn state(&self) -> Value {
        let s = self.shell();
        let b = s.browser();
        let u = s.ui_state();
        let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let mut selection: Vec<String> = b.selection.iter().map(name).collect();
        selection.sort();
        let groups: Vec<Value> =
            s.probe().groups.iter().map(|g| json!({"id": g.id, "panels": g.panels, "active": g.active, "rect": rect_json(g.rect)})).collect();
        json!({
            "workspace": u.workspace.id(),
            "view": format!("{:?}", u.view()),
            "folder": b.path.display().to_string(),
            "loading": b.loading,
            "visible": b.visible_assets().map(|a| a.name.clone()).collect::<Vec<_>>(),
            "selection": selection,
            "dialog": u.dialog.as_ref().map(|d| format!("{d:?}")),
            "dialogRect": s.probe().dialog.map(rect_json),
            "fullScreen": u.full_screen.as_ref().map(name),
            "toast": u.toast.as_ref().map(|t| t.0.clone()),
            "thumbSize": u.thumb_size,
            "panelsHidden": u.panels_hidden,
            "pointer": [self.pointer.x, self.pointer.y],
            "groups": groups,
            "launches": self.launches.json(),
        })
    }
}

pub(crate) fn rect_json(r: Rect) -> Value {
    json!([r.min.x, r.min.y, r.width(), r.height()])
}
