//! Pier's egui shell: the Bridge-style window, panels, workspaces, theme and interface strings.
//! All file-system work goes through `pier-engine`; this crate only draws and dispatches.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod bars;
mod content;
mod dialogs;
pub mod fonts;
pub mod i18n;
mod icons;
mod menu;
mod panels;
mod shell;
pub mod state;
pub mod theme;
pub mod workspace;

pub use shell::{Action, GroupProbe, PierShell, Probe};

/// Where Pier opens when no folder is given: Pictures, else the home folder, else the current directory.
pub fn default_start_folder() -> std::path::PathBuf {
    let home = pier_engine::fs::home_dir();
    home.as_ref()
        .map(|h| h.join("Pictures"))
        .filter(|p| p.is_dir())
        .or(home)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(pier_engine::fs::computer_root)
}
