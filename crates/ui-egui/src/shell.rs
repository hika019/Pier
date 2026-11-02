//! The window: event drain, actions, shortcuts and the workspace layout. Panels live in their own modules.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use egui::{CentralPanel, Color32, Context, Frame, Id, Key, KeyboardShortcut, Modifiers, Rect, Sense, TextureHandle, Ui, UiBuilder, Vec2, vec2};
use pier_engine::craft::{self, CraftApp};
use pier_engine::meta::MetaSummary;
use pier_engine::{Asset, AssetKind, Event, ImageSize, Worker};

use crate::i18n::{self, S, tr};
use crate::icons::{self, Icon};
use crate::state::{Browser, Dialog, Label, PrefPage, THUMB_MAX, THUMB_MIN, UiState, ViewMode};
use crate::theme::Tokens;
use crate::workspace::{Column, PanelId, Workspace};

/// Decoded thumbnails kept as textures; older ones off screen are released.
const MAX_THUMB_TEXTURES: usize = 1500;
const MAX_PREVIEW_TEXTURES: usize = 24;
/// How long a toast stays up, in seconds.
const TOAST_SECONDS: f64 = 2.5;
/// ⌘= / ⌘- change the thumbnail size by this many points.
pub(crate) const THUMB_STEP: f32 = 32.0;

pub(crate) enum Slot {
    Requested,
    Ready { tex: TextureHandle, size: Vec2, used: u64 },
    Failed,
}

/// Everything a menu item, button or shortcut can do. The commands of docs/command-catalog.md, first cut.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Open,
    OpenWith(Option<CraftApp>),
    OpenInCameraRaw,
    Reveal,
    CloseWindow,
    Quit,
    SortBy(crate::state::SortKey),
    ToggleDescending,
    SelectAll,
    DeselectAll,
    InvertSelection,
    SelectLabeled(bool),
    Rating(i8),
    RatingStep(i8),
    Label(Option<Label>),
    FullScreen,
    View(ViewMode),
    ToggleRejected,
    ToggleHidden,
    ToggleFolders,
    Refresh,
    Workspace(Workspace),
    ResetWorkspace,
    Preferences,
    Back,
    Forward,
    Up,
    GoTo(PathBuf),
    ThumbSize(f32),
    ThumbStep(f32),
    TogglePanels,
    Dialog(Dialog),
    /// Not built yet: says so in a toast.
    Soon(S),
}

/// What the last frame drew, for debugging and tests (the driver in `pier-ui-driver` reads it).
#[derive(Clone, Debug, Default)]
pub struct Probe {
    /// Every tabbed panel group, in drawing order.
    pub groups: Vec<GroupProbe>,
    /// The open modal dialog's frame, if any.
    pub dialog: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct GroupProbe {
    pub id: String,
    pub panels: Vec<&'static str>,
    pub active: &'static str,
    pub rect: Rect,
}

pub struct PierShell {
    pub(crate) browser: Browser,
    pub(crate) ui: UiState,
    pub(crate) tokens: Tokens,
    worker: Option<Worker>,
    pub(crate) images: HashMap<(PathBuf, ImageSize), Slot>,
    wanted: Vec<(PathBuf, ImageSize)>,
    wanted_set: HashSet<(PathBuf, ImageSize)>,
    last_wanted: HashSet<(PathBuf, ImageSize)>,
    pub(crate) meta: HashMap<PathBuf, Option<Box<MetaSummary>>>,
    pub(crate) subfolders: HashMap<PathBuf, Option<Result<Vec<PathBuf>, String>>>,
    pub(crate) craft_apps: HashMap<&'static str, Option<PathBuf>>,
    pub(crate) launcher: Option<PathBuf>,
    pub(crate) frame: u64,
    pub(crate) probe: Probe,
    pub(crate) native_menu: Option<craft_menu::NativeMenu>,
    pending: Vec<Action>,
    start: PathBuf,
    applied: Option<(crate::theme::Brightness, i18n::Lang)>,
    pub(crate) cjk_font: Option<&'static str>,
}

impl PierShell {
    pub fn new(start: PathBuf) -> Self {
        let craft_apps = craft::ALL.iter().map(|a| (a.id, craft::detect(*a))).collect();
        Self {
            browser: Browser::new(PathBuf::new()),
            ui: UiState::default(),
            tokens: Tokens::for_brightness(crate::theme::Brightness::Darkest),
            worker: None,
            images: HashMap::new(),
            wanted: Vec::new(),
            wanted_set: HashSet::new(),
            last_wanted: HashSet::new(),
            meta: HashMap::new(),
            subfolders: HashMap::new(),
            craft_apps,
            launcher: craft::detect_launcher(),
            frame: 0,
            probe: Probe::default(),
            native_menu: None,
            pending: Vec::new(),
            start,
            applied: None,
            cjk_font: None,
        }
    }

    pub fn probe(&self) -> &Probe {
        &self.probe
    }

    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    pub fn ui_state(&self) -> &UiState {
        &self.ui
    }

    pub fn browser_mut(&mut self) -> &mut Browser {
        &mut self.browser
    }

    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui
    }

    /// No folder listing, decode or metadata read is outstanding.
    pub fn is_idle(&self) -> bool {
        !self.browser.loading && self.worker.as_ref().is_none_or(Worker::is_idle) && !self.images.values().any(|s| matches!(s, Slot::Requested))
    }

    pub fn ui(&mut self, root: &mut Ui) {
        self.frame = self.frame.wrapping_add(1);
        let ctx = root.ctx().clone();
        self.ensure_started(&ctx);
        self.drain(&ctx);
        let menus = self.menu_model();
        self.handle_keys(&ctx, &menus);
        self.wanted.clear();
        self.probe.groups.clear();
        self.probe.dialog = None;
        self.wanted_set.clear();

        let t = self.tokens.clone();
        let bar = Frame::NONE.fill(t.bg_bar).inner_margin(egui::Margin::symmetric(6, 0));
        self.menu_ui(root, menus);
        egui::Panel::top("pier-app-bar").exact_size(t.bar_height).frame(bar).show(root, |ui| self.app_bar(ui));
        egui::Panel::top("pier-path-bar")
            .exact_size(t.path_bar_height)
            .frame(Frame::NONE.fill(t.bg_panel).inner_margin(egui::Margin::symmetric(6, 0)))
            .show(root, |ui| self.path_bar(ui));
        egui::Panel::bottom("pier-status").exact_size(t.status_height).frame(bar).show(root, |ui| self.status_bar(ui));
        CentralPanel::no_frame().frame(Frame::NONE.fill(t.bg_app).inner_margin(1)).show(root, |ui| self.workspace_ui(ui));

        self.full_screen_ui(&ctx);
        self.dialogs_ui(&ctx);
        self.toast_ui(&ctx);
        for action in std::mem::take(&mut self.pending) {
            self.run(&ctx, action);
        }
        self.sync_images();
    }

    fn ensure_started(&mut self, ctx: &Context) {
        let lang = self.ui.lang.resolve();
        i18n::set_lang(lang);
        if self.applied != Some((self.ui.brightness, lang)) {
            if self.applied.is_none() {
                self.cjk_font = crate::fonts::install_system_cjk(ctx);
            }
            self.tokens = Tokens::for_brightness(self.ui.brightness);
            self.tokens.apply(ctx);
            self.applied = Some((self.ui.brightness, lang));
        }
        if self.worker.is_none() {
            let c = ctx.clone();
            self.worker = Some(Worker::new(Arc::new(move || c.request_repaint())));
            let start = std::mem::take(&mut self.start);
            self.navigate(start, false);
        }
    }

    pub(crate) fn act(&mut self, action: Action) {
        self.pending.push(action);
    }

    pub(crate) fn soon(&mut self, what: S) {
        self.act(Action::Soon(what));
    }

    pub(crate) fn navigate(&mut self, path: PathBuf, record: bool) {
        if !self.browser.go_to(path.clone(), record) {
            return;
        }
        if let Some(w) = &self.worker {
            w.list_folder(self.browser.generation, path.clone());
        }
        for ancestor in path.ancestors() {
            self.ui.tree_expanded.insert(ancestor.to_path_buf());
            self.want_subfolders(ancestor);
        }
    }

    pub(crate) fn want_subfolders(&mut self, path: &Path) {
        if self.subfolders.contains_key(path) {
            return;
        }
        self.subfolders.insert(path.to_path_buf(), None);
        if let Some(w) = &self.worker {
            w.list_subfolders(path.to_path_buf());
        }
    }

    pub(crate) fn want_meta(&mut self, path: &Path) -> Option<&MetaSummary> {
        if !self.meta.contains_key(path) {
            self.meta.insert(path.to_path_buf(), None);
            if let Some(w) = &self.worker {
                w.read_meta(path.to_path_buf());
            }
        }
        self.meta.get(path).and_then(|m| m.as_deref())
    }

    /// The texture for `path` at `size` if decoded; asks for it otherwise.
    pub(crate) fn image(&mut self, path: &Path, size: ImageSize) -> Option<(egui::TextureId, Vec2)> {
        let key = (path.to_path_buf(), size);
        if self.wanted_set.insert(key.clone()) {
            self.wanted.push(key.clone());
        }
        match self.images.get_mut(&key) {
            Some(Slot::Ready { tex, size, used }) => {
                *used = self.frame;
                Some((tex.id(), *size))
            }
            _ => None,
        }
    }

    pub(crate) fn image_failed(&self, path: &Path, size: ImageSize) -> bool {
        matches!(self.images.get(&(path.to_path_buf(), size)), Some(Slot::Failed))
    }

    fn sync_images(&mut self) {
        let Some(worker) = &self.worker else { return };
        if self.wanted_set != self.last_wanted {
            worker.set_wanted(self.wanted_set.clone());
            self.last_wanted = self.wanted_set.clone();
        }
        // Newest requests decode first, so push in reverse to start with the top-left item.
        for key in self.wanted.iter().rev() {
            if !self.images.contains_key(key) {
                self.images.insert(key.clone(), Slot::Requested);
                worker.request_image(key.0.clone(), key.1);
            }
        }
        for (size, max) in [(ImageSize::Thumb, MAX_THUMB_TEXTURES), (ImageSize::Preview, MAX_PREVIEW_TEXTURES)] {
            let mut ready: Vec<(u64, (PathBuf, ImageSize))> = self
                .images
                .iter()
                .filter_map(|(k, s)| {
                    if let Slot::Ready { used, .. } = s
                        && k.1 == size
                    {
                        Some((*used, k.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            if ready.len() > max {
                ready.sort_by_key(|(used, _)| *used);
                let excess = ready.len() - max;
                for (_, key) in ready.into_iter().take(excess) {
                    if !self.wanted_set.contains(&key) {
                        self.images.remove(&key);
                    }
                }
            }
        }
    }

    fn drain(&mut self, ctx: &Context) {
        let Some(worker) = &self.worker else { return };
        let mut events = Vec::new();
        while let Some(e) = worker.try_recv() {
            events.push(e);
        }
        for event in events {
            match event {
                Event::Folder { generation, path, result } => {
                    if generation != self.browser.generation || path != self.browser.path {
                        continue;
                    }
                    self.browser.loading = false;
                    match result {
                        Ok(assets) => self.browser.assets = assets,
                        Err(e) => self.browser.error = Some(e),
                    }
                    self.browser.refresh_visible();
                }
                Event::Subfolders { path, result } => {
                    self.subfolders.insert(path, Some(result));
                }
                Event::Image { path, size, result } => {
                    let slot = match result {
                        Ok(img) => {
                            let color = egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &img.pixels);
                            let tex = ctx.load_texture(
                                path.to_string_lossy(),
                                color,
                                egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear)),
                            );
                            Slot::Ready { tex, size: vec2(img.width as f32, img.height as f32), used: self.frame }
                        }
                        Err(_) => Slot::Failed,
                    };
                    self.images.insert((path, size), slot);
                }
                Event::Skipped { path, size } => {
                    self.images.remove(&(path, size));
                }
                Event::Meta { path, summary } => {
                    self.meta.insert(path, Some(summary));
                }
            }
        }
    }

    fn handle_keys(&mut self, ctx: &Context, menus: &crate::menu::MenuModel) {
        if ctx.egui_wants_keyboard_input() || self.ui.dialog.is_some() || craft_menu::bar::is_open(ctx) {
            return;
        }
        for (shortcut, action) in crate::menu::shortcuts(menus) {
            if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                self.act(action);
            }
        }
        let (space, esc, enter, left, right, up, down, tab, shift) = ctx.input(|i| {
            (
                i.key_pressed(Key::Space),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::Tab),
                i.modifiers.shift,
            )
        });
        if self.ui.full_screen.is_some() {
            if space || esc {
                self.ui.full_screen = None;
            } else if left || right {
                self.step_full_screen(if right { 1 } else { -1 });
            }
            return;
        }
        if space {
            self.act(Action::FullScreen);
        }
        if enter {
            self.act(Action::Open);
        }
        if tab {
            self.act(Action::TogglePanels);
        }
        let cols = if self.ui.view() == ViewMode::Thumbnails { self.ui.grid_columns.max(1) as isize } else { 1 };
        let delta = match (left, right, up, down) {
            (true, ..) => -1,
            (_, true, ..) => 1,
            (_, _, true, _) => -cols,
            (.., true) => cols,
            _ => 0,
        };
        if delta != 0 && !self.browser.visible.is_empty() {
            let last = self.browser.visible.len() as isize - 1;
            let next = self.browser.anchor_index().map_or(0, |i| (i as isize + delta).clamp(0, last));
            let keep_anchor = self.browser.anchor.clone();
            self.browser.select_index(next as usize, shift);
            if shift {
                self.browser.anchor = keep_anchor;
            }
            self.ui.scroll_to_anchor = true;
        }
    }

    fn step_full_screen(&mut self, delta: isize) {
        let items: Vec<PathBuf> = self.browser.visible_assets().filter(|a| !a.is_folder()).map(|a| a.path.clone()).collect();
        let Some(cur) = self.ui.full_screen.clone() else { return };
        let Some(i) = items.iter().position(|p| *p == cur) else { return };
        let j = (i as isize + delta).clamp(0, items.len() as isize - 1) as usize;
        if let Some(p) = items.get(j) {
            self.ui.full_screen = Some(p.clone());
            self.browser.click_select(p, false, false);
        }
    }

    pub(crate) fn run(&mut self, ctx: &Context, action: Action) {
        match action {
            Action::Open => self.open_selection(None, false),
            Action::OpenWith(app) => self.open_selection(app, true),
            Action::OpenInCameraRaw => self.open_selection(Some(craft::LIGHTCRAFT), true),
            Action::Reveal => {
                if let Some(a) = self.browser.selected_assets().first() {
                    self.report(craft::reveal(&a.path));
                }
            }
            Action::CloseWindow | Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::SortBy(k) => self.refilter(|b| b.sort = Some(k)),
            Action::ToggleDescending => self.refilter(|b| b.descending = !b.descending),
            Action::SelectAll => self.browser.select_all(),
            Action::DeselectAll => self.browser.selection.clear(),
            Action::InvertSelection => self.browser.invert_selection(),
            Action::SelectLabeled(l) => self.browser.select_labeled(l),
            Action::Rating(r) => self.marks(|b| b.set_rating(r)),
            Action::RatingStep(d) => self.marks(|b| b.bump_rating(d)),
            Action::Label(l) => self.marks(|b| b.toggle_label(l)),
            Action::FullScreen => {
                self.ui.full_screen = self.browser.selected_assets().into_iter().find(|a| !a.is_folder()).map(|a| a.path.clone());
            }
            Action::View(v) => {
                self.ui.views.insert(self.ui.workspace, v);
            }
            Action::ToggleRejected => self.refilter(|b| b.show_rejected = !b.show_rejected),
            Action::ToggleHidden => self.refilter(|b| b.show_hidden = !b.show_hidden),
            Action::ToggleFolders => self.refilter(|b| b.show_folders = !b.show_folders),
            Action::Refresh => {
                let p = self.browser.path.clone();
                self.browser.assets.clear();
                self.images.clear();
                self.meta.clear();
                self.subfolders.clear();
                self.craft_apps = craft::ALL.iter().map(|a| (a.id, craft::detect(*a))).collect();
                self.launcher = craft::detect_launcher();
                self.browser.path = PathBuf::new();
                self.navigate(p, false);
            }
            Action::Workspace(w) => self.ui.workspace = w,
            Action::ResetWorkspace => {
                self.ui.views.remove(&self.ui.workspace);
                ctx.memory_mut(|m| m.reset_areas());
            }
            Action::Preferences => self.ui.dialog = Some(Dialog::Preferences(PrefPage::General)),
            Action::Back => {
                if let Some(p) = self.browser.back.pop() {
                    let cur = self.browser.path.clone();
                    self.navigate(p, false);
                    self.browser.forward.push(cur);
                }
            }
            Action::Forward => {
                if let Some(p) = self.browser.forward.pop() {
                    let cur = self.browser.path.clone();
                    self.navigate(p, false);
                    self.browser.back.push(cur);
                }
            }
            Action::Up => {
                if let Some(parent) = self.browser.path.parent().map(Path::to_path_buf) {
                    self.navigate(parent, true);
                }
            }
            Action::GoTo(p) => self.navigate(p, true),
            Action::ThumbSize(s) => self.ui.thumb_size = s.clamp(THUMB_MIN, THUMB_MAX),
            Action::ThumbStep(d) => self.ui.thumb_size = (self.ui.thumb_size + d).clamp(THUMB_MIN, THUMB_MAX),
            Action::TogglePanels => self.ui.panels_hidden = !self.ui.panels_hidden,
            Action::Dialog(d) => self.ui.dialog = Some(d),
            Action::Soon(s) => self.toast(format!("{} {}", tr(S::ComingSoonToast), tr(s).trim_end_matches('…'))),
        }
    }

    fn marks(&mut self, f: impl FnOnce(&mut Browser)) {
        f(&mut self.browser);
        self.browser.refresh_visible();
    }

    fn refilter(&mut self, f: impl FnOnce(&mut Browser)) {
        f(&mut self.browser);
        self.browser.refresh_visible();
    }

    pub(crate) fn toast(&mut self, text: String) {
        self.ui.toast = Some((text, f64::NAN));
    }

    fn report(&mut self, r: Result<(), String>) {
        if let Err(e) = r {
            self.toast(format!("{} {e}", tr(S::OpenFailed)));
        }
    }

    /// Opens the selection: a folder is browsed into; files go to their Craft App or the OS default.
    /// `app` set (Open With) forces the app; `forced` with `None` forces the OS default.
    fn open_selection(&mut self, app: Option<CraftApp>, forced: bool) {
        let selected: Vec<Asset> = self.browser.selected_assets().into_iter().cloned().collect();
        if let Some(folder) = selected.iter().find(|a| a.is_folder()) {
            self.navigate(folder.path.clone(), true);
            return;
        }
        let mut groups: Vec<(Option<CraftApp>, Vec<PathBuf>)> = Vec::new();
        for a in &selected {
            let target = if forced { app } else { craft::app_for(a.kind) };
            match groups.iter_mut().find(|(g, _)| *g == target) {
                Some((_, files)) => files.push(a.path.clone()),
                None => groups.push((target, vec![a.path.clone()])),
            }
        }
        for (target, files) in groups {
            match target {
                None => {
                    for f in &files {
                        let r = craft::open_default(f.as_os_str());
                        self.report(r);
                    }
                }
                Some(app) => match self.craft_apps.get(app.id).cloned().flatten() {
                    Some(path) => {
                        let r = craft::open_in_app(&path, &files);
                        self.report(r);
                    }
                    None => self.ui.dialog = Some(Dialog::CraftMissing { app, files }),
                },
            }
        }
    }

    fn workspace_ui(&mut self, ui: &mut Ui) {
        let ws = self.ui.workspace;
        if self.ui.panels_hidden {
            self.group(ui, &[PanelId::Content], "hidden");
            return;
        }
        let layout = ws.layout();
        for (i, col) in layout.left.iter().enumerate() {
            let id = format!("{}-left-{i}", ws.id());
            egui::Panel::left(id.clone()).resizable(true).default_size(col.width).min_size(140.0).frame(Frame::NONE).show(ui, |ui| self.column(ui, col, &id));
        }
        for (i, col) in layout.right.iter().enumerate() {
            let id = format!("{}-right-{i}", ws.id());
            egui::Panel::right(id.clone()).resizable(true).default_size(col.width).min_size(160.0).frame(Frame::NONE).show(ui, |ui| self.column(ui, col, &id));
        }
        let id = format!("{}-center", ws.id());
        CentralPanel::no_frame().show(ui, |ui| self.column(ui, &layout.center, &id));
    }

    fn column(&mut self, ui: &mut Ui, col: &Column, id: &str) {
        match col.groups {
            [first, rest @ ..] if !rest.is_empty() => {
                let h = ui.available_height();
                let top_id = format!("{id}-0");
                egui::Panel::top(top_id.clone())
                    .resizable(true)
                    .default_size(h * col.split)
                    .min_size(60.0)
                    .frame(Frame::NONE)
                    .show(ui, |ui| self.group(ui, first, &top_id));
                let bottom_id = format!("{id}-1");
                CentralPanel::no_frame().show(ui, |ui| {
                    for g in rest {
                        self.group(ui, g, &bottom_id);
                    }
                });
            }
            [only] => self.group(ui, only, &format!("{id}-0")),
            [] => {}
            _ => {}
        }
    }

    /// One tabbed panel group: the tab strip, then the active panel.
    fn group(&mut self, ui: &mut Ui, panels: &[PanelId], gid: &str) {
        let t = self.tokens.clone();
        // Claim exactly the space given and inset only the drawing: a resizable panel remembers the size
        // its contents claimed, so claiming less shrank it by a point every frame.
        let given = ui.available_rect_before_wrap();
        ui.allocate_rect(given, Sense::hover());
        let rect = given.shrink(1.0);
        let active_index = self.ui.tabs.get(gid).copied().unwrap_or(0).min(panels.len().saturating_sub(1));
        self.probe.groups.push(GroupProbe {
            id: gid.to_owned(),
            panels: panels.iter().map(|p| p.id()).collect(),
            active: panels.get(active_index).map_or("", |p| p.id()),
            rect,
        });
        let active = self.ui.tabs.get(gid).copied().unwrap_or(0).min(panels.len().saturating_sub(1));
        let strip = Rect::from_min_size(rect.min, vec2(rect.width(), t.tab_height));
        let painter = ui.painter_at(rect);
        painter.rect_filled(strip, 0.0, t.bg_tab_strip);
        let mut x = strip.left();
        for (i, p) in panels.iter().enumerate() {
            let galley = painter.layout_no_wrap(tr(p.title()).to_owned(), t.font(), if i == active { t.fg_strong } else { t.fg_dim });
            let tab = Rect::from_min_size(egui::pos2(x, strip.top()), vec2(galley.size().x + 22.0, t.tab_height));
            let resp = ui.interact(tab, Id::unique((gid, i)), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, i == active, tr(p.title())));
            if i == active {
                painter.rect_filled(tab, egui::CornerRadius { nw: 3, ne: 3, sw: 0, se: 0 }, if *p == PanelId::Content { t.bg_content } else { t.bg_panel });
            } else if resp.hovered() {
                painter.rect_filled(tab, 0.0, t.bg_cell_hover);
            }
            painter.galley(egui::pos2(tab.left() + 11.0, tab.center().y - galley.size().y / 2.0), galley, Color32::WHITE);
            if resp.clicked() {
                self.ui.tabs.insert(gid.to_owned(), i);
            }
            x = tab.right();
        }
        let menu_rect = Rect::from_center_size(egui::pos2(strip.right() - 12.0, strip.center().y), Vec2::splat(16.0));
        let mut menu_ui = ui.new_child(UiBuilder::new().max_rect(menu_rect));
        icons::menu_button(&mut menu_ui, &t, Icon::Menu, 14.0, tr(S::TipPanelMenu), |ui| {
            if ui.button(format!("{}  ({})", tr(S::Close), tr(S::ComingSoon))).clicked() {
                self.soon(S::Close);
            }
            if ui.button(tr(S::ResetWorkspace)).clicked() {
                self.act(Action::ResetWorkspace);
            }
        });

        let Some(&panel) = panels.get(active) else { return };
        let body = Rect::from_min_max(egui::pos2(rect.left(), strip.bottom()), rect.max);
        painter.rect_filled(body, 0.0, if panel == PanelId::Content { t.bg_content } else { t.bg_panel });
        let mut body_ui = ui.new_child(UiBuilder::new().max_rect(body).id_salt((gid, "body")));
        body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
        self.panel(&mut body_ui, panel);
    }

    fn panel(&mut self, ui: &mut Ui, panel: PanelId) {
        match panel {
            PanelId::Favorites => self.favorites_panel(ui),
            PanelId::Folders => self.folders_panel(ui),
            PanelId::Filter => self.filter_panel(ui),
            PanelId::Collections => self.collections_panel(ui),
            PanelId::Export => self.export_panel(ui),
            PanelId::Content => self.content_panel(ui),
            PanelId::Preview => self.preview_panel(ui),
            PanelId::Metadata => self.metadata_panel(ui),
            PanelId::Keywords => self.keywords_panel(ui),
            PanelId::Output => self.output_panel(ui),
            PanelId::OutputPreview => self.output_preview_panel(ui),
        }
    }

    fn toast_ui(&mut self, ctx: &Context) {
        let Some((text, since)) = self.ui.toast.clone() else { return };
        let now = ctx.input(|i| i.time);
        let since = if since.is_nan() {
            self.ui.toast = Some((text.clone(), now));
            now
        } else {
            since
        };
        if now - since > TOAST_SECONDS {
            self.ui.toast = None;
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
        let t = &self.tokens;
        egui::Area::new(Id::unique("pier-toast")).order(egui::Order::Tooltip).anchor(egui::Align2::CENTER_BOTTOM, vec2(0.0, -48.0)).interactable(false).show(
            ctx,
            |ui| {
                Frame::NONE.fill(t.bg_section).stroke(t.separator_stroke()).corner_radius(6).inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                    ui.label(egui::RichText::new(text).color(t.fg_strong));
                });
            },
        );
    }
}

pub(crate) fn mod_key(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}

pub(crate) fn reject_shortcut() -> KeyboardShortcut {
    // Bridge: Option+Delete on the Mac (the Backspace key), Alt+Delete elsewhere.
    KeyboardShortcut::new(Modifiers::ALT, if cfg!(target_os = "macos") { Key::Backspace } else { Key::Delete })
}

pub(crate) fn fmt_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u + 1 < UNITS.len() {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 { format!("{bytes} B") } else { format!("{v:.1} {}", UNITS.get(u).copied().unwrap_or("B")) }
}

pub(crate) fn fmt_time(t: Option<SystemTime>) -> String {
    t.map(|t| chrono::DateTime::<chrono::Local>::from(t).format("%Y/%m/%d %H:%M:%S").to_string()).unwrap_or_else(|| "—".to_owned())
}

pub(crate) fn kind_name(kind: AssetKind) -> S {
    match kind {
        AssetKind::Folder => S::KindFolder,
        AssetKind::Image => S::KindImage,
        AssetKind::Raw => S::KindRaw,
        AssetKind::Layered => S::KindLayered,
        AssetKind::Vector => S::KindVector,
        AssetKind::Layout => S::KindLayout,
        AssetKind::Pdf => S::KindPdf,
        AssetKind::Video => S::KindVideo,
        AssetKind::Audio => S::KindAudio,
        AssetKind::Font => S::KindFont,
        AssetKind::Document => S::KindDocument,
        AssetKind::Other => S::KindOther,
    }
}

pub(crate) fn label_color(t: &Tokens, l: Label) -> Color32 {
    match l {
        Label::Select => t.labels.select,
        Label::Second => t.labels.second,
        Label::Approved => t.labels.approved,
        Label::Review => t.labels.review,
        Label::Todo => t.labels.todo,
    }
}

pub(crate) fn reveal_label() -> S {
    if cfg!(target_os = "macos") {
        S::RevealInFinder
    } else if cfg!(windows) {
        S::RevealInExplorer
    } else {
        S::RevealInFileManager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_bridge() {
        assert_eq!(fmt_size(512), "512 B");
        assert_eq!(fmt_size(45 * 1024 * 1024 + 200 * 1024), "45.2 MB");
    }

    #[test]
    fn every_string_has_both_languages() {
        for s in S::ALL {
            assert!(!s.get(i18n::Lang::En).is_empty(), "{s:?}");
            assert!(!s.get(i18n::Lang::Ja).is_empty(), "{s:?}");
        }
    }

    #[test]
    fn shortcuts_do_not_collide() {
        for lang in [i18n::Lang::En, i18n::Lang::Ja] {
            i18n::set_lang(lang);
            let shell = PierShell::new(PathBuf::from("/"));
            let list = crate::menu::shortcuts(&shell.menu_model());
            for (i, (a, _)) in list.iter().enumerate() {
                for (b, _) in list.iter().skip(i + 1) {
                    assert!(a != b, "{a:?} is bound twice");
                }
            }
        }
    }
}
