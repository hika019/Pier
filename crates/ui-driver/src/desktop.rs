//! A desktop with chosen Craft Apps "installed" that records what Pier asks it to start, instead of
//! starting anything.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pier_engine::craft::{CraftApp, Desktop, Launch};
use serde_json::{Value, json};

#[derive(Clone, Default)]
pub struct Launches(Arc<Mutex<Vec<Launch>>>);

impl Launches {
    pub fn take(&self) -> Vec<Launch> {
        self.0.lock().map(|mut l| std::mem::take(&mut *l)).unwrap_or_default()
    }

    pub fn all(&self) -> Vec<Launch> {
        self.0.lock().map(|l| l.clone()).unwrap_or_default()
    }

    /// `{"app": "PhotoCraft", "files": ["a.CR3"]}`, `{"default": "..."}` or `{"reveal": "..."}`.
    pub fn json(&self) -> Value {
        let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let stem = |p: &PathBuf| p.file_stem().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Value::Array(
            self.all()
                .iter()
                .map(|l| match l {
                    Launch::App { app, files } => json!({"app": stem(app), "files": files.iter().map(name).collect::<Vec<_>>()}),
                    Launch::Default(t) => json!({"default": t.to_string_lossy()}),
                    Launch::Reveal(p) => json!({"reveal": name(p)}),
                })
                .collect(),
        )
    }
}

/// The Craft Apps the fake desktop reports, shared so a test can install one while Pier runs.
#[derive(Clone, Default)]
pub struct Installed(Arc<Mutex<Vec<CraftApp>>>);

impl Installed {
    pub fn new(apps: Vec<CraftApp>) -> Self {
        Self(Arc::new(Mutex::new(apps)))
    }

    pub fn install(&self, app: CraftApp) {
        if let Ok(mut apps) = self.0.lock() {
            apps.push(app);
        }
    }

    fn contains(&self, app: CraftApp) -> bool {
        self.0.lock().is_ok_and(|apps| apps.contains(&app))
    }
}

/// Where the fake desktop says ArtCraft Launcher is, when it says it is installed.
pub const FAKE_LAUNCHER: &str = "/Applications/ArtCraft Launcher.app";

pub struct FakeDesktop {
    installed: Installed,
    launcher: bool,
    launches: Launches,
}

impl FakeDesktop {
    pub fn new(installed: Installed, launcher: bool, launches: Launches) -> Self {
        Self { installed, launcher, launches }
    }
}

impl Desktop for FakeDesktop {
    fn detect(&self, app: CraftApp) -> Option<PathBuf> {
        self.installed.contains(app).then(|| PathBuf::from(format!("/Applications/{}.app", app.name)))
    }

    fn detect_launcher(&self) -> Option<PathBuf> {
        self.launcher.then(|| PathBuf::from(FAKE_LAUNCHER))
    }

    fn launch(&mut self, launch: Launch) -> Result<(), String> {
        self.launches.0.lock().map_err(|e| e.to_string())?.push(launch);
        Ok(())
    }
}
