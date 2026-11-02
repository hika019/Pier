//! Drives Pier offscreen from a JSON script, printing a log and writing PNGs and state JSON.
//!
//! ```sh
//! cargo run -p pier-ui-driver --bin pier-drive -- --steps '[{"op":"screenshot","path":"shot.png"}]'
//! cargo run -p pier-ui-driver --bin pier-drive -- --script scenario.json --lang ja --out-dir target/pier-drive
//! ```
//! Options: `--script <file>` or `--steps <json>`, `--dir <folder>` (default: a fresh demo folder),
//! `--out-dir <dir>` (default `target/pier-drive`), `--workspace <id>`, `--lang en|ja`,
//! `--theme darkest|dark|light|lightest`, `--size 1440x900`, `--scale 1`.
//! Ends with the window state as JSON. Exits 1 when a step fails (an assertion, a missing label).

use std::path::PathBuf;
use std::process::ExitCode;

use pier_ui_driver::{Config, Driver, script};
use pier_ui_egui::i18n::Lang;
use pier_ui_egui::theme::Brightness;
use pier_ui_egui::workspace::Workspace;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pier-drive: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let text = match (arg(&args, "--script"), arg(&args, "--steps")) {
        (Some(p), _) => std::fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"))?,
        (None, Some(s)) => s,
        (None, None) => "[]".to_owned(),
    };
    let steps = script::parse(&text)?;
    let config = Config {
        dir: arg(&args, "--dir").map(PathBuf::from),
        size: arg(&args, "--size")
            .and_then(|s| s.split_once('x').and_then(|(w, h)| Some(egui::vec2(w.parse().ok()?, h.parse().ok()?))))
            .unwrap_or(egui::vec2(1440.0, 900.0)),
        scale: arg(&args, "--scale").and_then(|s| s.parse().ok()).unwrap_or(1.0),
        workspace: arg(&args, "--workspace").and_then(|w| Workspace::from_id(&w)).unwrap_or(Workspace::Essentials),
        lang: if arg(&args, "--lang").as_deref() == Some("ja") { Lang::Ja } else { Lang::En },
        theme: match arg(&args, "--theme").as_deref() {
            Some("dark") => Brightness::Dark,
            Some("light") => Brightness::Light,
            Some("lightest") => Brightness::Lightest,
            _ => Brightness::Darkest,
        },
    };
    let out_dir = PathBuf::from(arg(&args, "--out-dir").unwrap_or_else(|| "target/pier-drive".into()));
    let mut driver = Driver::new(config)?;
    println!("folder: {}", driver.dir.display());
    let result = script::run(&mut driver, &steps, &out_dir);
    match &result {
        Ok(log) => log.iter().for_each(|l| println!("{l}")),
        Err(e) => println!("FAILED {e}"),
    }
    println!("{}", serde_json::to_string_pretty(&driver.state()).map_err(|e| e.to_string())?);
    result.map(|_| ())
}
