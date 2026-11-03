# Pier

**English** | [日本語](README.ja.md)

[![CI](https://github.com/hika019/Pier/actions/workflows/ci.yml/badge.svg)](https://github.com/hika019/Pier/actions/workflows/ci.yml)

**Pier is a clean-room, pure-Rust digital asset browser in the spirit of Adobe Bridge.**
Browse folders as thumbnails, preview, rate, label, add keywords, edit metadata, batch-rename, and organise with
collections and stacks, then hand files to [storytold's Craft Apps](https://github.com/storytold) for editing
(PhotoCraft, LightCraft, VectorCraft, DesignCraft, PdfCraft, FilmCraft and more).

- Pier **assumes the Craft Apps are installed** and works with them in place of Photoshop, Camera Raw, Illustrator
  and InDesign. When an app is missing, it points you to [ArtCraft Launcher](https://github.com/storytold/craft-launcher)
  to install it.
- Layout and conventions follow [storytold/photocraft](https://github.com/storytold/photocraft): a native
  egui/eframe + wgpu UI with no webview, Rust 1.95.0 pinned (photocraft's minimum, 1.95), edition 2024, and
  [craft-libs](https://github.com/storytold/craft-libs) at a fixed revision.

> Status: **early prototype.** The window is laid out like Bridge's. Working: folder browsing, thumbnails, preview,
> metadata, sorting, filters, ratings and labels (for the session only), and opening files in a Craft App (images
> and camera raws go to PhotoCraft, as Bridge sends them to Photoshop; raws open through Camera Raw first). Other
> menus and panels are in place and say "coming soon". On macOS the menus live in the system menu bar. English and
> Japanese (Preferences › Advanced › Language, or `PIER_LOCALE=ja`).

## Running

```sh
cargo run -p pier                 # opens your Pictures folder
cargo run -p pier -- ~/Photos     # opens a folder of your choice
cargo run -p pier-ui-driver --bin pier-drive -- --lang ja --steps '[{"op":"screenshot","path":"shot.png"}]'   # drive and capture offscreen (docs/debugging.md)
cargo nextest run --workspace     # the tests, as CI runs them (doctests: cargo test --doc --workspace)
```

`rust-toolchain.toml` selects Rust 1.95.0; rustup installs it on first use.

GitHub Actions runs the tests on every push: tests on macOS and Ubuntu, fmt and clippy on Ubuntu, and a build
check on Windows ([.github/workflows/ci.yml](.github/workflows/ci.yml)).

## Documentation

The design documents are written in Japanese.

| Document | What it covers |
|---|---|
| [AGENTS.md](AGENTS.md) | Rules for contributors and AI agents (read first) |
| [CLAUDE.md](CLAUDE.md) | How agents work in this repository: lead and subagents, git, review, tests |
| [dashboard.md](dashboard.md) | Decisions waiting on the maintainer (empty is normal) |
| [.claude/skills/](.claude/skills/) | Task procedures: bug-frontier, session-recovery, parity-observation (observing Bridge), ui-verify |
| [docs/overview.md](docs/overview.md) | Product overview, goals, scope and terms |
| [docs/requirements.md](docs/requirements.md) | Functional and non-functional requirements, with IDs and priorities |
| [docs/architecture.md](docs/architecture.md) | Architecture: workspace layout, layers, threads and data flow |
| [docs/data-model.md](docs/data-model.md) | Data model, cache database and how metadata is written |
| [docs/ui-design.md](docs/ui-design.md) | Screen design: workspaces, panels and dialogs |
| [docs/command-catalog.md](docs/command-catalog.md) | Menus, command IDs and shortcuts |
| [docs/file-formats.md](docs/file-formats.md) | Supported formats and how thumbnails and previews are made |
| [docs/craft-integration.md](docs/craft-integration.md) | Working with the Craft Apps, and guiding installation |
| [docs/testing.md](docs/testing.md) | Test strategy and performance budgets |
| [docs/debugging.md](docs/debugging.md) | The debugging environment, usable by AI too: offscreen input, PNGs and state JSON |
| [docs/roadmap.md](docs/roadmap.md) | Milestones and their exit criteria |
| [docs/decisions.md](docs/decisions.md) | Architecture decision records (ADRs) |

## License

[MIT](LICENSE). Adobe, Adobe Bridge and Photoshop are trademarks of Adobe Inc. Pier is not affiliated with Adobe
and contains no Adobe code, icons or assets (see the clean-room rules in [AGENTS.md](AGENTS.md)).
