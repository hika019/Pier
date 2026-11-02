//! The menu bar as data (craft-menu): one tree of commands, shown as the macOS menu bar or drawn in the
//! window on Windows and Linux. Command ids follow docs/command-catalog.md; the shortcuts here are the
//! app's only shortcut table.

use std::collections::HashMap;

use craft_keymap::{Key as ChordKey, Mods};
use craft_menu::{Chord, CommandRef, Item, ItemRole, LayoutOptions, Menu, MenuBar, MenuRole, Node};
use egui::{Context, Key, KeyboardShortcut, Ui};
use pier_engine::craft;

use crate::i18n::{self, S, tr};
use crate::shell::{Action, PierShell};
use crate::state::{Label, SortKey, ViewMode};
use crate::workspace::Workspace;

pub(crate) const APP_NAME: &str = "Pier";

const CMD: Mods = Mods::CMD;
const CMD_SHIFT: Mods = Mods { cmd: true, shift: true, ..Mods::NONE };
const CMD_ALT: Mods = Mods { cmd: true, alt: true, ..Mods::NONE };

fn chord(mods: Mods, key: Key) -> Option<Chord> {
    Some(Chord { mods, key: ChordKey::from_egui(key)? })
}

/// Bridge rejects with Option+Delete on the Mac (the Backspace key) and Alt+Delete elsewhere.
fn reject_chord() -> Option<Chord> {
    chord(Mods::ALT, if cfg!(target_os = "macos") { Key::Backspace } else { Key::Delete })
}

/// The menu tree and what each command id does.
pub(crate) struct MenuModel {
    pub bar: MenuBar,
    pub actions: HashMap<String, Action>,
}

#[derive(Default)]
struct Builder {
    actions: HashMap<String, Action>,
}

impl Builder {
    fn item(&mut self, id: &str, label: &str, shortcut: Option<Chord>, action: Action) -> Node {
        self.actions.insert(id.to_owned(), action);
        let mut item = Item::new(CommandRef::new(id), label);
        if let Some(c) = shortcut {
            item = item.shortcut(c);
        }
        Node::Item(item)
    }

    fn cmd(&mut self, id: &str, label: S, shortcut: Option<Chord>, action: Action) -> Node {
        self.item(id, tr(label), shortcut, action)
    }

    /// Not built yet: labelled as coming soon; choosing it says so.
    fn soon(&mut self, id: &str, label: S, shortcut: Option<Chord>) -> Node {
        self.item(id, &format!("{}{}", tr(label), tr(S::SoonSuffix)), shortcut, Action::Soon(label))
    }

    fn check(&mut self, id: &str, label: S, checked: bool, action: Action) -> Node {
        self.actions.insert(id.to_owned(), action);
        Node::Item(Item::new(CommandRef::new(id), tr(label)).check(checked))
    }

    fn radio(&mut self, id: &str, label: &str, selected: bool, shortcut: Option<Chord>, action: Action) -> Node {
        self.actions.insert(id.to_owned(), action);
        let mut item = Item::new(CommandRef::new(id), label).radio(selected);
        if let Some(c) = shortcut {
            item = item.shortcut(c);
        }
        Node::Item(item)
    }

    fn role(node: Node, role: ItemRole) -> Node {
        match node {
            Node::Item(item) => Node::Item(item.role(role)),
            other => other,
        }
    }
}

impl PierShell {
    pub(crate) fn menu_model(&self) -> MenuModel {
        let mut b = Builder::default();
        let file = self.file_menu(&mut b);
        let edit = self.edit_menu(&mut b);
        let view = self.view_menu(&mut b);
        let stacks = Menu::new(
            tr(S::MenuStacks),
            vec![
                b.soon("stack.group", S::GroupAsStack, chord(CMD, Key::G)),
                b.soon("stack.ungroup", S::Ungroup, chord(CMD_SHIFT, Key::G)),
                b.soon("stack.open", S::OpenStack, None),
                b.soon("stack.close", S::CloseStack, None),
                b.soon("stack.promoteToTop", S::PromoteToTop, None),
                b.soon("stack.expandAll", S::ExpandAllStacks, None),
                b.soon("stack.collapseAll", S::CollapseAllStacks, None),
                Node::Separator,
                b.soon("stack.autoStack", S::AutoStack, None),
            ],
        );
        let label = Self::label_menu(&mut b);
        let tools = Self::tools_menu(&mut b);
        let window = self.window_menu(&mut b);
        let help = Menu::new(
            tr(S::MenuHelp),
            vec![
                b.soon("help.open", S::PierHelp, None),
                b.soon("help.shortcuts", S::KeyboardShortcuts, None),
                Node::Separator,
                b.cmd("help.getCraftApps", S::GetCraftApps, None, Action::Dialog(crate::state::Dialog::CraftApps)),
                b.soon("help.checkForUpdates", S::CheckForUpdates, None),
                Node::Separator,
                Builder::role(b.cmd("help.about", S::AboutPier, None, Action::Dialog(crate::state::Dialog::About)), ItemRole::About),
            ],
        )
        .role(MenuRole::Help);
        let bar = MenuBar { menus: vec![file, edit, view, stacks, label, tools, window, help], documents: Vec::new() };
        MenuModel { bar, actions: b.actions }
    }

    fn file_menu(&self, b: &mut Builder) -> Menu {
        let mut open_with = vec![b.cmd("file.openWith.default", S::DefaultApp, None, Action::OpenWith(None)), Node::Separator];
        for app in craft::ALL {
            let installed = self.craft_apps.get(app.id).is_some_and(Option::is_some);
            let text = if installed { app.name.to_owned() } else { format!("{}  ({})", app.name, tr(S::NotInstalled)) };
            open_with.push(b.item(&format!("file.openWith.{}", app.id), &text, None, Action::OpenWith(Some(app))));
        }
        Menu::new(
            tr(S::MenuFile),
            vec![
                b.soon("window.new", S::NewWindow, chord(CMD, Key::N)),
                b.soon("file.newFolder", S::NewFolder, chord(CMD_SHIFT, Key::N)),
                b.cmd("file.open", S::Open, chord(CMD, Key::O), Action::Open),
                Node::submenu(tr(S::OpenWith), open_with),
                b.soon("file.openRecent", S::OpenRecent, None),
                b.cmd("file.openInCameraRaw", S::OpenInCameraRaw, chord(CMD, Key::R), Action::OpenInCameraRaw),
                Node::Separator,
                b.cmd("window.close", S::CloseWindow, chord(CMD, Key::W), Action::CloseWindow),
                Node::Separator,
                b.soon("file.trash", S::MoveToTrash, chord(CMD, if cfg!(target_os = "macos") { Key::Backspace } else { Key::Delete })),
                Node::Separator,
                b.cmd("file.reveal", crate::shell::reveal_label(), None, Action::Reveal),
                b.soon("file.getPhotosFromCamera", S::GetPhotosFromCamera, None),
                Node::Separator,
                b.soon("file.moveTo", S::MoveTo, None),
                b.soon("file.copyTo", S::CopyTo, None),
                b.soon("file.place", S::Place, None),
                Node::Separator,
                b.soon("favorites.add", S::AddToFavorites, None),
                Node::Separator,
                b.soon("file.info", S::FileInfo, chord(CMD_ALT, Key::I)),
                Node::Separator,
                Builder::role(b.cmd("app.quit", S::Quit, chord(CMD, Key::Q), Action::Quit), ItemRole::Quit),
            ],
        )
        .role(MenuRole::File)
    }

    fn edit_menu(&self, b: &mut Builder) -> Menu {
        Menu::new(
            tr(S::MenuEdit),
            vec![
                b.soon("edit.undo", S::Undo, chord(CMD, Key::Z)),
                b.soon("edit.redo", S::Redo, chord(CMD_SHIFT, Key::Z)),
                Node::Separator,
                b.soon("edit.cut", S::Cut, None),
                b.soon("edit.copy", S::Copy, None),
                b.soon("edit.paste", S::Paste, None),
                b.soon("edit.duplicate", S::Duplicate, chord(CMD, Key::D)),
                Node::Separator,
                b.cmd("select.all", S::SelectAll, chord(CMD, Key::A), Action::SelectAll),
                b.cmd("select.none", S::DeselectAll, chord(CMD_SHIFT, Key::A), Action::DeselectAll),
                b.cmd("select.invert", S::InvertSelection, chord(CMD_SHIFT, Key::I), Action::InvertSelection),
                b.cmd("select.labeled", S::SelectLabeled, None, Action::SelectLabeled(true)),
                b.cmd("select.unlabeled", S::SelectUnlabeled, None, Action::SelectLabeled(false)),
                Node::Separator,
                b.soon("search.find", S::Find, chord(CMD, Key::F)),
                Node::Separator,
                b.soon("edit.developSettings", S::DevelopSettings, None),
                b.soon("edit.rotate.180", S::Rotate180, None),
                b.soon("edit.rotate.cw90", S::RotateCw, chord(CMD, Key::CloseBracket)),
                b.soon("edit.rotate.ccw90", S::RotateCcw, chord(CMD, Key::OpenBracket)),
                Node::Separator,
                Builder::role(b.cmd("app.preferences", S::Preferences, chord(CMD, Key::K), Action::Preferences), ItemRole::Settings),
            ],
        )
        .role(MenuRole::Edit)
    }

    fn view_menu(&self, b: &mut Builder) -> Menu {
        let view = self.ui.view();
        let br = &self.browser;
        let mut sort: Vec<Node> = SortKey::ALL
            .into_iter()
            .map(|k| b.radio(&format!("view.sort.{k:?}").to_lowercase(), tr(k.name()), br.sort == Some(k), None, Action::SortBy(k)))
            .collect();
        sort.push(Node::Separator);
        sort.push(b.check("view.sort.ascending", S::Ascending, !br.descending, Action::ToggleDescending));
        Menu::new(
            tr(S::MenuView),
            vec![
                b.cmd("view.fullScreenPreview", S::FullScreenPreview, chord(Mods::NONE, Key::Space), Action::FullScreen),
                b.soon("view.slideshow", S::Slideshow, chord(CMD, Key::L)),
                b.soon("view.slideshowOptions", S::SlideshowOptions, chord(CMD_SHIFT, Key::L)),
                b.soon("view.reviewMode", S::ReviewMode, chord(CMD, Key::B)),
                Node::Separator,
                b.radio("view.asThumbnails", tr(S::AsThumbnails), view == ViewMode::Thumbnails, None, Action::View(ViewMode::Thumbnails)),
                b.radio("view.asDetails", tr(S::AsDetails), view == ViewMode::Details, None, Action::View(ViewMode::Details)),
                b.radio("view.asList", tr(S::AsList), view == ViewMode::List, None, Action::View(ViewMode::List)),
                Node::Separator,
                b.soon("view.thumbnailOnly", S::ShowThumbnailOnly, None),
                b.soon("view.gridLock", S::GridLock, None),
                Node::Separator,
                b.check("view.showRejected", S::ShowRejected, br.show_rejected, Action::ToggleRejected),
                b.check("view.showHidden", S::ShowHidden, br.show_hidden, Action::ToggleHidden),
                b.check("view.showFolders", S::ShowFolders, br.show_folders, Action::ToggleFolders),
                b.soon("view.showSubfolderItems", S::ShowSubfolderItems, None),
                Node::Separator,
                Node::submenu(tr(S::Sort), sort),
                b.cmd("view.refresh", S::Refresh, chord(Mods::NONE, Key::F5), Action::Refresh),
                Node::Separator,
                b.cmd("view.thumbSize.inc", S::TipLarger, chord(CMD, Key::Equals), Action::ThumbStep(crate::shell::THUMB_STEP)),
                b.cmd("view.thumbSize.dec", S::TipSmaller, chord(CMD, Key::Minus), Action::ThumbStep(-crate::shell::THUMB_STEP)),
            ],
        )
        .role(MenuRole::View)
    }

    fn label_menu(b: &mut Builder) -> Menu {
        let mut v = vec![
            b.cmd("label.rating.0", S::NoRating, chord(CMD, Key::Num0), Action::Rating(0)),
            b.cmd("label.rating.reject", S::Reject, reject_chord(), Action::Rating(-1)),
        ];
        for (n, key) in (1..=5).zip([Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5]) {
            v.push(b.item(&format!("label.rating.{n}"), &"★".repeat(n as usize), chord(CMD, key), Action::Rating(n)));
        }
        v.push(Node::Separator);
        v.push(b.cmd("label.rating.dec", S::DecreaseRating, chord(CMD, Key::Comma), Action::RatingStep(-1)));
        v.push(b.cmd("label.rating.inc", S::IncreaseRating, chord(CMD, Key::Period), Action::RatingStep(1)));
        v.push(Node::Separator);
        v.push(b.cmd("label.color.none", S::NoLabel, None, Action::Label(None)));
        for (l, key) in Label::ALL.into_iter().zip([Some(Key::Num6), Some(Key::Num7), Some(Key::Num8), Some(Key::Num9), None]) {
            v.push(b.cmd(&format!("label.color.{l:?}").to_lowercase(), l.name(), key.and_then(|k| chord(CMD, k)), Action::Label(Some(l))));
        }
        Menu::new(tr(S::MenuLabel), v)
    }

    fn tools_menu(b: &mut Builder) -> Menu {
        Menu::new(
            tr(S::MenuTools),
            vec![
                b.soon("tools.batchRename", S::BatchRename, chord(CMD_SHIFT, Key::R)),
                Node::Separator,
                b.soon("meta.template.create", S::CreateMetadataTemplate, None),
                b.soon("meta.template.append", S::AppendMetadata, None),
                b.soon("meta.template.replace", S::ReplaceMetadata, None),
                Node::Separator,
                Node::submenu(
                    tr(S::Cache),
                    vec![b.soon("cache.buildAndExport", S::BuildAndExportCache, None), b.soon("cache.purgeFolder", S::PurgeCache, None)],
                ),
                Node::Separator,
                Node::submenu(
                    "PhotoCraft",
                    vec![
                        b.soon("craft.photocraft.batch", S::PcBatch, None),
                        b.soon("craft.photocraft.imageProcessor", S::PcImageProcessor, None),
                        b.soon("craft.photocraft.loadAsLayers", S::PcLoadAsLayers, None),
                        b.soon("craft.photocraft.mergeHdr", S::PcMergeHdr, None),
                        b.soon("craft.photocraft.photomerge", S::PcPhotomerge, None),
                    ],
                ),
                Node::submenu(
                    "LightCraft",
                    vec![b.soon("craft.lightcraft.import", S::LcImport, None), b.soon("craft.lightcraft.convertToDng", S::LcDng, None)],
                ),
                Node::submenu("VectorCraft", vec![b.soon("craft.vectorcraft.imageTrace", S::VcImageTrace, None)]),
            ],
        )
    }

    fn window_menu(&self, b: &mut Builder) -> Menu {
        let fkeys = [Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6];
        let mut ws: Vec<Node> = Workspace::ALL
            .into_iter()
            .enumerate()
            .map(|(i, w)| {
                b.radio(
                    &format!("workspace.select.{}", w.id()),
                    tr(w.name()),
                    self.ui.workspace == w,
                    fkeys.get(i).and_then(|k| chord(CMD, *k)),
                    Action::Workspace(w),
                )
            })
            .collect();
        ws.push(Node::Separator);
        ws.push(b.soon("workspace.save", S::NewWorkspace, None));
        ws.push(b.cmd("workspace.reset", S::ResetWorkspace, None, Action::ResetWorkspace));
        let mut v = vec![Node::submenu(tr(S::Workspace), ws), Node::Separator];
        for (id, s) in [
            ("favorites", S::PFavorites),
            ("folders", S::PFolders),
            ("content", S::PContent),
            ("preview", S::PPreview),
            ("metadata", S::PMetadata),
            ("keywords", S::PKeywords),
            ("filter", S::PFilter),
            ("collections", S::PCollections),
            ("export", S::PExport),
            ("output", S::POutput),
            ("pathBar", S::PathBar),
        ] {
            v.push(b.soon(&format!("panel.toggle.{id}"), s, None));
        }
        Menu::new(tr(S::MenuWindow), v).role(MenuRole::Window)
    }

    /// The menus: the macOS menu bar when installed, else drawn at the top of the window.
    pub(crate) fn menu_ui(&mut self, ui: &mut Ui, model: MenuModel) {
        let ctx = ui.ctx().clone();
        let gate = craft_menu::Gate { modal: self.ui.dialog.is_some(), recording_shortcut: false, text_editing: ctx.egui_wants_keyboard_input() };
        let label = |english: &'static str| i18n::layout_label(english);
        let options = LayoutOptions { label: &label, ..LayoutOptions::new(APP_NAME) };
        let chosen = if let Some(native) = self.native_menu.as_mut() {
            native.set_gate(gate);
            let clicks = native.take_clicks(&ctx);
            native.sync(craft_menu::mac_layout(&model.bar, &options));
            clicks
        } else {
            let laid_out = craft_menu::layout(&model.bar, craft_menu::Target::current(), &options).bar;
            let t = self.tokens.clone();
            let mut chosen = Vec::new();
            egui::Panel::top("pier-menu").frame(egui::Frame::NONE.fill(t.bg_app).inner_margin(egui::Margin::symmetric(4, 1))).show(ui, |ui| {
                chosen = craft_menu::MenuBarUi::new(&laid_out).gate(gate).label(&label).show(ui);
            });
            chosen
        };
        for command in chosen {
            if let Some(action) = model.actions.get(&command.id) {
                self.act(action.clone());
            }
        }
    }

    /// Installs the macOS menu bar (once, from eframe's creator). Elsewhere, or when AppKit refuses, the
    /// menus stay in the window.
    pub fn install_native_menu(&mut self, ctx: &Context) {
        #[cfg(target_os = "macos")]
        {
            i18n::set_lang(self.ui.lang.resolve());
            let model = self.menu_model();
            let label = |english: &'static str| i18n::layout_label(english);
            let options = LayoutOptions { label: &label, ..LayoutOptions::new(APP_NAME) };
            self.native_menu = craft_menu::mac::install(ctx, &craft_menu::mac_layout(&model.bar, &options).bar);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = ctx;
    }

    /// Before egui sees a frame's input: the native menu's key equivalents become key presses (eframe's
    /// `raw_input_hook`).
    pub fn raw_input(&mut self, raw: &mut egui::RawInput) {
        if let Some(native) = self.native_menu.as_mut() {
            native.raw_input(raw);
        }
    }

    pub fn has_native_menu(&self) -> bool {
        self.native_menu.is_some()
    }
}

/// Every shortcut: the menu items' (one table) plus the navigation keys that have no menu item.
/// Plain Space is left out: the window handles it itself (full screen on, and off again).
pub(crate) fn shortcuts(model: &MenuModel) -> Vec<(KeyboardShortcut, Action)> {
    fn walk(nodes: &[Node], model: &MenuModel, out: &mut Vec<(KeyboardShortcut, Action)>) {
        for n in nodes {
            match n {
                Node::Item(item) => {
                    if let (Some(c), Some(a)) = (&item.shortcut, model.actions.get(&item.command.id))
                        && !(c.mods.is_none() && c.key == ChordKey::Space)
                    {
                        out.push((c.to_egui_shortcut(), a.clone()));
                    }
                }
                Node::Submenu { children, .. } => walk(children, model, out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for m in &model.bar.menus {
        walk(&m.children, model, &mut out);
    }
    out.push((KeyboardShortcut::new(egui::Modifiers::COMMAND, Key::ArrowDown), Action::Open));
    out.push((KeyboardShortcut::new(egui::Modifiers::COMMAND, Key::ArrowUp), Action::Up));
    // With Shift first, so ⇧⌘A isn't read as ⌘A.
    out.sort_by_key(|(s, _)| std::cmp::Reverse(s.modifiers.shift as u8 + s.modifiers.alt as u8));
    out
}
