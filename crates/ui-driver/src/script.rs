//! JSON scripts for [`Driver`]: an array of steps, each an object with an `op`.
//!
//! ```json
//! [
//!   {"op": "click", "label": "Filmstrip"},
//!   {"op": "markLayout"},
//!   {"op": "wiggle", "from": [100, 400], "to": [1400, 400], "steps": 60},
//!   {"op": "assertLayoutStable"},
//!   {"op": "click", "at": [600, 220]},
//!   {"op": "key", "key": "5", "mods": ["cmd"]},
//!   {"op": "screenshot", "path": "after.png"},
//!   {"op": "state", "path": "after.json"}
//! ]
//! ```
//!
//! Ops: `move {to}`, `wiggle {from, to, steps}`, `click {at | label, button?, mods?}`,
//! `doubleClick {at | label}`, `rightClick {at | label}`, `drag {from, to, steps?}`,
//! `scroll {at, dx?, dy}`, `key {key, mods?}` (egui key names: `A`, `Space`, `ArrowDown`, `F5`, `5`),
//! `type {text}`, `waitIdle`, `frames {n}`, `resize {size}`, `screenshot {path}`, `state {path?}`,
//! `widgets {path?, contains?}`, `markLayout`, `assertLayoutStable {tolerance?}`,
//! `assert {path, equals}` (a JSON pointer into the state, such as `/workspace`).

use std::path::{Path, PathBuf};

use egui::{Key, Modifiers, PointerButton, Rect, pos2, vec2};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::driver::{Driver, rect_json};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Step {
    Move {
        to: [f32; 2],
    },
    Wiggle {
        from: [f32; 2],
        to: [f32; 2],
        #[serde(default = "default_steps")]
        steps: usize,
    },
    Click {
        at: Option<[f32; 2]>,
        label: Option<String>,
        #[serde(default)]
        button: Button,
        #[serde(default)]
        mods: Vec<String>,
    },
    DoubleClick {
        at: Option<[f32; 2]>,
        label: Option<String>,
    },
    RightClick {
        at: Option<[f32; 2]>,
        label: Option<String>,
    },
    Drag {
        from: [f32; 2],
        to: [f32; 2],
        #[serde(default = "default_steps")]
        steps: usize,
    },
    Scroll {
        at: [f32; 2],
        #[serde(default)]
        dx: f32,
        dy: f32,
    },
    Key {
        key: String,
        #[serde(default)]
        mods: Vec<String>,
    },
    Type {
        text: String,
    },
    WaitIdle,
    Frames {
        n: usize,
    },
    Resize {
        size: [f32; 2],
    },
    Screenshot {
        path: String,
    },
    State {
        path: Option<String>,
    },
    Widgets {
        path: Option<String>,
        contains: Option<String>,
    },
    MarkLayout,
    AssertLayoutStable {
        #[serde(default)]
        tolerance: f32,
    },
    Assert {
        path: String,
        equals: Value,
    },
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Middle,
}

fn default_steps() -> usize {
    30
}

pub fn parse(text: &str) -> Result<Vec<Step>, String> {
    serde_json::from_str(text).map_err(|e| format!("script: {e}"))
}

fn mods(names: &[String]) -> Result<Modifiers, String> {
    let mut m = Modifiers::NONE;
    for n in names {
        match n.to_ascii_lowercase().as_str() {
            // ⌘ on macOS, Ctrl elsewhere, like the app's shortcuts.
            "cmd" | "command" | "mod" => m |= Modifiers::COMMAND,
            "ctrl" | "control" => m |= Modifiers::CTRL,
            "alt" | "option" => m |= Modifiers::ALT,
            "shift" => m |= Modifiers::SHIFT,
            other => return Err(format!("unknown modifier {other:?} (cmd, ctrl, alt, shift)")),
        }
    }
    Ok(m)
}

fn key(name: &str) -> Result<Key, String> {
    Key::from_name(name).ok_or_else(|| format!("unknown key {name:?} (egui key names: A, Space, ArrowDown, F5, 5, …)"))
}

/// Runs `steps`, writing files under `out_dir`. Returns one log line per step; stops at the first
/// failed step with an error that names it.
pub fn run(driver: &mut Driver, steps: &[Step], out_dir: &Path) -> Result<Vec<String>, String> {
    let mut log = Vec::new();
    let mut marked: Option<Vec<(String, Rect)>> = None;
    for (i, step) in steps.iter().enumerate() {
        let line = run_step(driver, step, out_dir, &mut marked).map_err(|e| format!("step {} ({step:?}): {e}", i + 1))?;
        log.push(format!("{:>3}. {line}", i + 1));
    }
    Ok(log)
}

fn target(driver: &Driver, at: Option<[f32; 2]>, label: &Option<String>) -> Result<egui::Pos2, String> {
    match (at, label) {
        (Some([x, y]), _) => Ok(pos2(x, y)),
        (None, Some(l)) => driver.find(l, false).or_else(|| driver.find(l, true)).map(|w| w.rect.center()).ok_or_else(|| format!("no widget labelled {l:?}")),
        (None, None) => Err("give `at` or `label`".into()),
    }
}

fn write(out_dir: &Path, path: &str, value: &Value) -> Result<PathBuf, String> {
    let p = out_dir.join(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&p, serde_json::to_string_pretty(value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(p)
}

fn run_step(d: &mut Driver, step: &Step, out_dir: &Path, marked: &mut Option<Vec<(String, Rect)>>) -> Result<String, String> {
    Ok(match step {
        Step::Move { to } => {
            d.move_to(pos2(to[0], to[1]));
            format!("move {to:?}")
        }
        Step::Wiggle { from, to, steps } => {
            d.move_path(pos2(from[0], from[1]), pos2(to[0], to[1]), *steps);
            format!("wiggle {from:?} → {to:?} in {steps} frames")
        }
        Step::Click { at, label, button, mods: m } => {
            let p = target(d, *at, label)?;
            let b = match button {
                Button::Primary => PointerButton::Primary,
                Button::Secondary => PointerButton::Secondary,
                Button::Middle => PointerButton::Middle,
            };
            d.click_at(p, b, mods(m)?);
            format!("click {:?} at [{:.0}, {:.0}]", label.as_deref().unwrap_or(""), p.x, p.y)
        }
        Step::DoubleClick { at, label } => {
            let p = target(d, *at, label)?;
            d.double_click_at(p);
            format!("double-click at [{:.0}, {:.0}]", p.x, p.y)
        }
        Step::RightClick { at, label } => {
            let p = target(d, *at, label)?;
            d.click_at(p, PointerButton::Secondary, Modifiers::NONE);
            format!("right-click at [{:.0}, {:.0}]", p.x, p.y)
        }
        Step::Drag { from, to, steps } => {
            d.drag(pos2(from[0], from[1]), pos2(to[0], to[1]), *steps);
            format!("drag {from:?} → {to:?}")
        }
        Step::Scroll { at, dx, dy } => {
            d.scroll_at(pos2(at[0], at[1]), vec2(*dx, *dy));
            format!("scroll {dy} at {at:?}")
        }
        Step::Key { key: k, mods: m } => {
            d.key(key(k)?, mods(m)?);
            format!("key {m:?}+{k}")
        }
        Step::Type { text } => {
            d.type_text(text);
            format!("type {text:?}")
        }
        Step::WaitIdle => {
            if !d.settle() {
                return Err("still busy after the timeout".into());
            }
            "idle".into()
        }
        Step::Frames { n } => {
            d.frames(*n);
            format!("{n} frames")
        }
        Step::Resize { size } => {
            d.resize(vec2(size[0], size[1]));
            format!("resize {size:?}")
        }
        Step::Screenshot { path } => {
            let p = out_dir.join(path);
            d.screenshot(&p)?;
            format!("screenshot {}", p.display())
        }
        Step::State { path } => {
            let state = d.state();
            match path {
                Some(path) => format!("state {}", write(out_dir, path, &state)?.display()),
                None => format!("state {state}"),
            }
        }
        Step::Widgets { path, contains } => {
            let list: Vec<Value> = d
                .widgets()
                .into_iter()
                .filter(|w| contains.as_ref().is_none_or(|c| w.label.contains(c.as_str())))
                .map(|w| json!({"role": w.role, "label": w.label, "rect": rect_json(w.rect)}))
                .collect();
            let n = list.len();
            match path {
                Some(path) => format!("{n} widgets {}", write(out_dir, path, &Value::Array(list))?.display()),
                None => format!("{n} widgets {}", Value::Array(list)),
            }
        }
        Step::MarkLayout => {
            *marked = Some(d.layout());
            "layout marked".into()
        }
        Step::AssertLayoutStable { tolerance } => {
            let before = marked.as_ref().ok_or("markLayout first")?;
            let now = d.layout();
            let moved: Vec<String> = before
                .iter()
                .filter_map(|(id, r)| match now.iter().find(|(nid, _)| nid == id) {
                    None => Some(format!("{id} disappeared")),
                    Some((_, nr)) if (nr.min - r.min).length() > *tolerance || (nr.size() - r.size()).length() > *tolerance => {
                        Some(format!("{id}: {:?} → {:?}", rect_json(*r), rect_json(*nr)))
                    }
                    _ => None,
                })
                .collect();
            if !moved.is_empty() {
                return Err(format!("panels moved: {}", moved.join("; ")));
            }
            format!("layout stable ({} groups)", now.len())
        }
        Step::Assert { path, equals } => {
            let state = d.state();
            let got = state.pointer(path).cloned().unwrap_or(Value::Null);
            if &got != equals {
                return Err(format!("{path} is {got}, expected {equals}"));
            }
            format!("{path} == {equals}")
        }
    })
}
