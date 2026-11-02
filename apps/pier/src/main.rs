//! Pier desktop app: `pier [folder]`.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
// No console window behind the app on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;

use pier_ui_egui::PierShell;

struct App(PierShell);

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.0.raw_input(raw_input);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}

fn main() -> eframe::Result {
    let start = std::env::args_os().nth(1).map(PathBuf::from).filter(|p| p.is_dir()).unwrap_or_else(pier_ui_egui::default_start_folder);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Pier").with_inner_size([1440.0, 900.0]).with_min_inner_size([820.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Pier",
        options,
        Box::new(move |cc| {
            let mut shell = PierShell::new(start);
            // The macOS menu bar; elsewhere the menus are drawn in the window.
            shell.install_native_menu(&cc.egui_ctx);
            eprintln!("pier: menus {}", if shell.has_native_menu() { "in the macOS menu bar" } else { "in the window" });
            Ok(Box::new(App(shell)))
        }),
    )
}
