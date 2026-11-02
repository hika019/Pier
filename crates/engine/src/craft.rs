//! Handing files to the Craft Apps (docs/craft-integration.md), the OS default app, and the file manager.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::AssetKind;

/// A Craft App Pier can hand files to. `id` is the machine name ArtCraft Launcher uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CraftApp {
    pub id: &'static str,
    pub name: &'static str,
}

pub const PHOTOCRAFT: CraftApp = CraftApp { id: "photocraft", name: "PhotoCraft" };
pub const LIGHTCRAFT: CraftApp = CraftApp { id: "lightcraft", name: "LightCraft" };
pub const VECTORCRAFT: CraftApp = CraftApp { id: "vectorcraft", name: "VectorCraft" };
pub const DESIGNCRAFT: CraftApp = CraftApp { id: "designcraft", name: "DesignCraft" };
pub const PDFCRAFT: CraftApp = CraftApp { id: "pdfcraft", name: "PdfCraft" };
pub const FILMCRAFT: CraftApp = CraftApp { id: "filmcraft", name: "FilmCraft" };
pub const SOUNDCRAFT: CraftApp = CraftApp { id: "soundcraft", name: "SoundCraft" };

pub const ALL: [CraftApp; 7] = [PHOTOCRAFT, LIGHTCRAFT, VECTORCRAFT, DESIGNCRAFT, PDFCRAFT, FILMCRAFT, SOUNDCRAFT];

pub const LAUNCHER_NAME: &str = "ArtCraft Launcher";
pub const LAUNCHER_DOWNLOAD_URL: &str = "https://github.com/storytold/craft-launcher/releases/latest";

impl CraftApp {
    pub fn page_url(self) -> String {
        format!("https://getartcraft.com/apps/{}", self.id)
    }
}

/// The default association: which Craft App opens a kind (Open, double-click), or `None` for the
/// OS default app. Bridge keeps its own association table rather than following the OS, and opens
/// images and camera raws in Photoshop (raws through Camera Raw first); PhotoCraft does both
/// (docs/craft-integration.md §1).
pub fn app_for(kind: AssetKind) -> Option<CraftApp> {
    match kind {
        AssetKind::Image | AssetKind::Raw | AssetKind::Layered => Some(PHOTOCRAFT),
        AssetKind::Vector => Some(VECTORCRAFT),
        AssetKind::Layout => Some(DESIGNCRAFT),
        AssetKind::Pdf => Some(PDFCRAFT),
        AssetKind::Video => Some(FILMCRAFT),
        AssetKind::Audio => Some(SOUNDCRAFT),
        _ => None,
    }
}

/// Where an installed copy lives, in ArtCraft Launcher's search order (docs/craft-integration.md §2; first cut).
pub fn detect(app: CraftApp) -> Option<PathBuf> {
    candidates(app.name, app.id).into_iter().find(|p| p.exists())
}

pub fn detect_launcher() -> Option<PathBuf> {
    candidates(LAUNCHER_NAME, "artcraft-launcher").into_iter().find(|p| p.exists())
}

fn candidates(name: &str, id: &str) -> Vec<PathBuf> {
    let home = crate::fs::home_dir();
    let mut v = Vec::new();
    if cfg!(target_os = "macos") {
        v.push(PathBuf::from(format!("/Applications/{name}.app")));
        if let Some(h) = &home {
            v.push(h.join(format!("Applications/{name}.app")));
        }
    } else if cfg!(windows) {
        v.push(PathBuf::from(format!("C:\\Program Files\\ArtCraft\\{name}\\{id}.exe")));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            v.push(PathBuf::from(local).join(format!("Programs\\ArtCraft\\{name}\\{id}.exe")));
        }
    } else {
        v.push(PathBuf::from(format!("/usr/bin/{id}")));
        v.push(PathBuf::from(format!("/usr/local/bin/{id}")));
        if let Some(h) = &home {
            v.push(h.join(format!(".local/bin/{id}")));
        }
    }
    v
}

/// Opens `files` in the app at `app_path` (an `.app` bundle on macOS, else the executable).
pub fn open_in_app(app_path: &Path, files: &[PathBuf]) -> Result<(), String> {
    let mut cmd = if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg("-a").arg(app_path);
        c
    } else {
        Command::new(app_path)
    };
    cmd.args(files);
    spawn(cmd)
}

/// Opens a file, folder or URL with the system's default handler.
pub fn open_default(target: &std::ffi::OsStr) -> Result<(), String> {
    let mut cmd = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(windows) {
        Command::new("explorer")
    } else {
        Command::new("xdg-open")
    };
    cmd.arg(target);
    spawn(cmd)
}

/// Shows `path` selected in Finder / Explorer (its folder elsewhere).
pub fn reveal(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg("-R").arg(path);
        spawn(c)
    } else if cfg!(windows) {
        let mut c = Command::new("explorer");
        c.arg(format!("/select,{}", path.display()));
        spawn(c)
    } else {
        open_default(path.parent().unwrap_or(path).as_os_str())
    }
}

/// What Pier asked the desktop to start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Launch {
    App { app: PathBuf, files: Vec<PathBuf> },
    Default(std::ffi::OsString),
    Reveal(PathBuf),
}

/// The desktop Pier runs on: where the Craft Apps are, and how other programs are started.
/// [`System`] is the real one; tests substitute a fake that records each [`Launch`].
pub trait Desktop: Send {
    fn detect(&self, app: CraftApp) -> Option<PathBuf>;
    fn detect_launcher(&self) -> Option<PathBuf>;
    fn launch(&mut self, launch: Launch) -> Result<(), String>;
}

pub struct System;

impl Desktop for System {
    fn detect(&self, app: CraftApp) -> Option<PathBuf> {
        detect(app)
    }

    fn detect_launcher(&self) -> Option<PathBuf> {
        detect_launcher()
    }

    fn launch(&mut self, launch: Launch) -> Result<(), String> {
        match launch {
            Launch::App { app, files } => open_in_app(&app, &files),
            Launch::Default(target) => open_default(&target),
            Launch::Reveal(path) => reveal(&path),
        }
    }
}

/// Arguments go as an array, never through a shell (docs/requirements.md NFR-S4). The child is
/// reaped on a thread so it never lingers as a zombie.
fn spawn(mut cmd: Command) -> Result<(), String> {
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bridge opens these in Photoshop (raws through Camera Raw); the PhotoCraft column is the
    /// same table with Photoshop replaced (docs/craft-integration.md §1).
    #[test]
    fn images_and_raws_open_in_photocraft_like_bridge_opens_them_in_photoshop() {
        for ext in ["jpg", "JPEG", "png", "tif", "tiff", "gif", "webp", "bmp", "cr3", "CR2", "nef", "arw", "dng", "raf", "orf", "rw2", "psd", "psb"] {
            assert_eq!(app_for(AssetKind::from_extension(ext)), Some(PHOTOCRAFT), "{ext}");
        }
    }
}
