//! Drives the real Pier shell offscreen with real input events, for scenario tests and for AI agents
//! debugging the UI: move and click the mouse, drag, scroll, press keys, click widgets by their
//! accessible label, wait for background work, then read the state as JSON and the screen as PNG.
//!
//! - [`Driver`]: the Rust API, used by `tests/scenarios.rs`.
//! - [`script`]: the same operations as JSON steps, run by the `pier-drive` binary.
//! - [`demo`]: a demo folder of original, procedurally drawn images.

pub mod demo;
mod driver;
pub mod script;

pub use driver::{Config, Driver, Widget};
