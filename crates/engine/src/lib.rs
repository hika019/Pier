//! Pier's headless core. Nothing here uses egui, so the UI stays a thin client.
//!
//! First cut: folder listing, thumbnail decoding, EXIF summaries and Craft App hand-off.
//! It will be split into the crates of docs/architecture.md (core, fs, decode, render, craft-bridge,
//! engine) as each part grows.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod craft;
pub mod decode;
pub mod fs;
pub mod meta;
pub mod model;
pub mod worker;

pub use model::{Asset, AssetKind};
pub use worker::{Event, ImageSize, Worker};
