//! What the shell shows: the folder being browsed, selection, sort, filters, session marks and UI state.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use pier_engine::{Asset, AssetKind};

use crate::i18n::{LangSetting, S};
use crate::theme::Brightness;
use crate::workspace::Workspace;

/// Bridge's five labels, in menu order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Label {
    Select,
    Second,
    Approved,
    Review,
    Todo,
}

impl Label {
    pub const ALL: [Self; 5] = [Self::Select, Self::Second, Self::Approved, Self::Review, Self::Todo];

    pub fn name(self) -> S {
        match self {
            Self::Select => S::LabelSelect,
            Self::Second => S::LabelSecond,
            Self::Approved => S::LabelApproved,
            Self::Review => S::LabelReview,
            Self::Todo => S::LabelTodo,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortKey {
    Name,
    Type,
    Created,
    Modified,
    Size,
    Rating,
    Label,
}

impl SortKey {
    pub const ALL: [Self; 7] = [Self::Name, Self::Type, Self::Created, Self::Modified, Self::Size, Self::Rating, Self::Label];

    pub fn name(self) -> S {
        match self {
            Self::Name => S::SortName,
            Self::Type => S::SortType,
            Self::Created => S::SortCreated,
            Self::Modified => S::SortModified,
            Self::Size => S::SortSize,
            Self::Rating => S::SortRating,
            Self::Label => S::SortLabel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ViewMode {
    #[default]
    Thumbnails,
    Details,
    List,
}

/// The path bar's star menu (Filter items by rating).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QuickFilter {
    #[default]
    All,
    Unrated,
    AtLeast(i8),
    Labeled,
    Unlabeled,
    RejectedOnly,
}

/// The Filter panel's choices. Within a category any value matches (OR); categories combine with AND.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub ratings: BTreeSet<i8>,
    pub labels: BTreeSet<Option<Label>>,
    pub kinds: BTreeSet<AssetKind>,
    pub dates: BTreeSet<String>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.ratings.is_empty() && self.labels.is_empty() && self.kinds.is_empty() && self.dates.is_empty()
    }
}

pub fn date_key(asset: &Asset) -> Option<String> {
    asset.modified.map(|t| chrono::DateTime::<chrono::Local>::from(t).format("%Y/%m/%d").to_string())
}

/// Folder browsing: listing, history, selection and the session's ratings and labels.
#[derive(Default)]
pub struct Browser {
    pub path: PathBuf,
    pub generation: u64,
    pub loading: bool,
    pub error: Option<String>,
    pub assets: Vec<Asset>,
    /// Indices into `assets` after filtering and sorting.
    pub visible: Vec<usize>,
    pub selection: HashSet<PathBuf>,
    pub anchor: Option<PathBuf>,
    pub back: Vec<PathBuf>,
    pub forward: Vec<PathBuf>,
    pub recent: Vec<PathBuf>,
    pub ratings: HashMap<PathBuf, i8>,
    pub labels: HashMap<PathBuf, Label>,
    pub sort: Option<SortKey>,
    pub descending: bool,
    pub quick: QuickFilter,
    pub filter: Filter,
    pub search: String,
    pub show_hidden: bool,
    pub show_rejected: bool,
    pub show_folders: bool,
}

/// Recent folders kept for the application bar's menu.
const MAX_RECENT: usize = 10;

impl Browser {
    pub fn new(path: PathBuf) -> Self {
        Self { path, sort: Some(SortKey::Name), show_rejected: true, show_folders: true, ..Self::default() }
    }

    pub fn rating(&self, p: &Path) -> i8 {
        self.ratings.get(p).copied().unwrap_or(0)
    }

    pub fn label(&self, p: &Path) -> Option<Label> {
        self.labels.get(p).copied()
    }

    /// Moves to `path`, recording history. Returns false when already there.
    pub fn go_to(&mut self, path: PathBuf, record: bool) -> bool {
        if path == self.path && !self.assets.is_empty() {
            return false;
        }
        if record && !self.path.as_os_str().is_empty() {
            self.back.push(std::mem::take(&mut self.path));
            self.forward.clear();
        }
        self.recent.retain(|p| p != &path);
        self.recent.insert(0, path.clone());
        self.recent.truncate(MAX_RECENT);
        self.path = path;
        self.generation = self.generation.wrapping_add(1);
        self.loading = true;
        self.error = None;
        self.assets.clear();
        self.visible.clear();
        self.selection.clear();
        self.anchor = None;
        self.filter = Filter::default();
        true
    }

    pub fn visible_assets(&self) -> impl Iterator<Item = &Asset> {
        self.visible.iter().filter_map(|&i| self.assets.get(i))
    }

    pub fn visible_at(&self, i: usize) -> Option<&Asset> {
        self.visible.get(i).and_then(|&j| self.assets.get(j))
    }

    /// The selection in display order.
    pub fn selected_assets(&self) -> Vec<&Asset> {
        self.visible_assets().filter(|a| self.selection.contains(&a.path)).collect()
    }

    /// Before the Filter panel: hidden files, rejects, folders, the quick filter and the search text.
    pub fn passes_base(&self, a: &Asset) -> bool {
        if a.hidden && !self.show_hidden {
            return false;
        }
        if a.is_folder() && !self.show_folders {
            return false;
        }
        let rating = self.rating(&a.path);
        if rating < 0 && !self.show_rejected && self.quick != QuickFilter::RejectedOnly {
            return false;
        }
        let label = self.label(&a.path);
        let quick = match self.quick {
            QuickFilter::All => true,
            QuickFilter::Unrated => rating == 0,
            QuickFilter::AtLeast(n) => rating >= n,
            QuickFilter::Labeled => label.is_some(),
            QuickFilter::Unlabeled => label.is_none(),
            QuickFilter::RejectedOnly => rating < 0,
        };
        quick && (self.search.is_empty() || a.name.to_lowercase().contains(&self.search.to_lowercase()))
    }

    fn passes_filter(&self, a: &Asset) -> bool {
        let f = &self.filter;
        (f.ratings.is_empty() || f.ratings.contains(&self.rating(&a.path)))
            && (f.labels.is_empty() || f.labels.contains(&self.label(&a.path)))
            && (f.kinds.is_empty() || f.kinds.contains(&a.kind))
            && (f.dates.is_empty() || date_key(a).is_some_and(|d| f.dates.contains(&d)))
    }

    pub fn refresh_visible(&mut self) {
        let mut v: Vec<usize> = (0..self.assets.len()).filter(|&i| self.assets.get(i).is_some_and(|a| self.passes_base(a) && self.passes_filter(a))).collect();
        let key = self.sort.unwrap_or(SortKey::Name);
        v.sort_by(|&i, &j| match (self.assets.get(i), self.assets.get(j)) {
            (Some(a), Some(b)) => {
                let folders_first = b.is_folder().cmp(&a.is_folder());
                let ord = self.compare(a, b, key).then_with(|| natural_cmp(&a.name, &b.name));
                folders_first.then(if self.descending { ord.reverse() } else { ord })
            }
            _ => Ordering::Equal,
        });
        self.visible = v;
        let shown: HashSet<PathBuf> = self.visible_assets().map(|a| a.path.clone()).collect();
        self.selection.retain(|p| shown.contains(p));
    }

    fn compare(&self, a: &Asset, b: &Asset, key: SortKey) -> Ordering {
        match key {
            SortKey::Name => natural_cmp(&a.name, &b.name),
            SortKey::Type => a.ext.cmp(&b.ext),
            SortKey::Created => a.created.cmp(&b.created),
            SortKey::Modified => a.modified.cmp(&b.modified),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::Rating => self.rating(&a.path).cmp(&self.rating(&b.path)),
            SortKey::Label => self.label(&a.path).cmp(&self.label(&b.path)),
        }
    }

    /// Click selection: plain replaces, `toggle` (⌘/Ctrl) adds or removes, `range` (Shift) extends from the anchor.
    pub fn click_select(&mut self, path: &Path, toggle: bool, range: bool) {
        if range && let Some(anchor) = self.anchor.clone() {
            let order: Vec<PathBuf> = self.visible_assets().map(|a| a.path.clone()).collect();
            let (Some(i), Some(j)) = (order.iter().position(|p| *p == anchor), order.iter().position(|p| p == path)) else { return };
            let (lo, hi) = (i.min(j), i.max(j));
            if !toggle {
                self.selection.clear();
            }
            self.selection.extend(order.get(lo..=hi).unwrap_or_default().iter().cloned());
            return;
        }
        if toggle {
            if !self.selection.remove(path) {
                self.selection.insert(path.to_path_buf());
            }
        } else {
            self.selection.clear();
            self.selection.insert(path.to_path_buf());
        }
        self.anchor = Some(path.to_path_buf());
    }

    pub fn select_index(&mut self, i: usize, extend: bool) {
        let Some(path) = self.visible_at(i).map(|a| a.path.clone()) else { return };
        if extend {
            self.click_select(&path, false, true);
            return;
        }
        self.selection.clear();
        self.selection.insert(path.clone());
        self.anchor = Some(path);
    }

    /// Index of the anchor (the item keyboard navigation moves from).
    pub fn anchor_index(&self) -> Option<usize> {
        let anchor = self.anchor.as_ref()?;
        self.visible_assets().position(|a| &a.path == anchor)
    }

    pub fn set_rating(&mut self, rating: i8) {
        for p in self.selection.clone() {
            if rating == 0 {
                self.ratings.remove(&p);
            } else {
                self.ratings.insert(p, rating.clamp(-1, 5));
            }
        }
    }

    pub fn bump_rating(&mut self, delta: i8) {
        for p in self.selection.clone() {
            let r = (self.rating(&p).max(0) + delta).clamp(0, 5);
            if r == 0 {
                self.ratings.remove(&p);
            } else {
                self.ratings.insert(p, r);
            }
        }
    }

    /// Bridge toggles: applying the label an item already has removes it.
    pub fn toggle_label(&mut self, label: Option<Label>) {
        let all_have = label.is_some() && self.selection.iter().all(|p| self.label(p) == label);
        for p in self.selection.clone() {
            match label {
                Some(l) if !all_have => {
                    self.labels.insert(p, l);
                }
                _ => {
                    self.labels.remove(&p);
                }
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selection = self.visible_assets().map(|a| a.path.clone()).collect();
    }

    pub fn invert_selection(&mut self) {
        let all: HashSet<PathBuf> = self.visible_assets().map(|a| a.path.clone()).collect();
        self.selection = all.difference(&self.selection).cloned().collect();
    }

    pub fn select_labeled(&mut self, labeled: bool) {
        self.selection = self.visible_assets().filter(|a| self.label(&a.path).is_some() == labeled).map(|a| a.path.clone()).collect();
    }

    pub fn selected_size(&self) -> u64 {
        self.selected_assets().iter().map(|a| a.size).sum()
    }
}

/// Compares names with runs of digits by value, so `IMG_2` sorts before `IMG_10`.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut s = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        s.push(c);
                        it.next();
                    }
                    s
                };
                let (m, n) = (take(&mut x), take(&mut y));
                let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                let ord = m.len().cmp(&n.len()).then_with(|| m.cmp(n));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(c), Some(d)) => {
                let ord = c.to_lowercase().cmp(d.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                x.next();
                y.next();
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    Preferences(PrefPage),
    CraftMissing { app: pier_engine::craft::CraftApp, files: Vec<PathBuf> },
    CraftApps,
    About,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefPage {
    General,
    Advanced,
    Applications,
    Cache,
    CacheManagement,
    Content,
    Export,
    FileTypes,
    Interface,
    Keywords,
    LabelsRatings,
    MediaCache,
    Metadata,
    Output,
    Playback,
    QuickActions,
    StartupScripts,
    Thumbnails,
    Workflow,
}

impl PrefPage {
    /// Bridge 2026's order on macOS: General and Advanced, then A–Z by the English name
    /// (docs/ui-design.md §6.1).
    pub const ALL: [Self; 19] = [
        Self::General,
        Self::Advanced,
        Self::Applications,
        Self::Cache,
        Self::CacheManagement,
        Self::Content,
        Self::Export,
        Self::FileTypes,
        Self::Interface,
        Self::Keywords,
        Self::LabelsRatings,
        Self::MediaCache,
        Self::Metadata,
        Self::Output,
        Self::Playback,
        Self::QuickActions,
        Self::StartupScripts,
        Self::Thumbnails,
        Self::Workflow,
    ];

    pub fn name(self) -> S {
        match self {
            Self::General => S::PrefGeneral,
            Self::Advanced => S::PrefAdvanced,
            Self::Applications => S::PrefApplications,
            Self::Cache => S::PrefCache,
            Self::CacheManagement => S::PrefCacheManagement,
            Self::Content => S::PrefContent,
            Self::Export => S::PrefExport,
            Self::FileTypes => S::PrefFileTypes,
            Self::Interface => S::PrefInterface,
            Self::Keywords => S::PrefKeywords,
            Self::LabelsRatings => S::PrefLabelsRatings,
            Self::MediaCache => S::PrefMediaCache,
            Self::Metadata => S::PrefMetadata,
            Self::Output => S::PrefOutput,
            Self::Playback => S::PrefPlayback,
            Self::QuickActions => S::PrefQuickActions,
            Self::StartupScripts => S::PrefStartupScripts,
            Self::Thumbnails => S::PrefThumbnails,
            Self::Workflow => S::PrefWorkflow,
        }
    }
}

/// The settings being edited in Preferences: OK keeps them, Cancel and Escape drop them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefDraft {
    pub brightness: Brightness,
    pub lang: LangSetting,
}

/// Window state that is not about the folder: workspace, theme, language, dialogs.
pub struct UiState {
    pub workspace: Workspace,
    pub brightness: Brightness,
    pub lang: LangSetting,
    pub thumb_size: f32,
    pub views: HashMap<Workspace, ViewMode>,
    /// Active tab per panel group, keyed by the group's id.
    pub tabs: HashMap<String, usize>,
    pub dialog: Option<Dialog>,
    /// Open while Preferences is.
    pub pref_draft: Option<PrefDraft>,
    pub toast: Option<(String, f64)>,
    pub full_screen: Option<PathBuf>,
    pub collapsed_sections: HashSet<S>,
    pub tree_expanded: HashSet<PathBuf>,
    pub panels_hidden: bool,
    /// Columns of the thumbnail grid last frame, for ↑/↓.
    pub grid_columns: usize,
    pub scroll_to_anchor: bool,
}

pub const THUMB_MIN: f32 = 64.0;
pub const THUMB_MAX: f32 = 400.0;

impl Default for UiState {
    fn default() -> Self {
        Self {
            workspace: Workspace::Essentials,
            brightness: Brightness::Darkest,
            lang: LangSetting::Auto,
            thumb_size: 128.0,
            views: HashMap::new(),
            tabs: HashMap::new(),
            dialog: None,
            pref_draft: None,
            toast: None,
            full_screen: None,
            collapsed_sections: HashSet::new(),
            tree_expanded: HashSet::new(),
            panels_hidden: false,
            grid_columns: 1,
            scroll_to_anchor: false,
        }
    }
}

impl UiState {
    pub fn view(&self) -> ViewMode {
        self.views.get(&self.workspace).copied().unwrap_or_else(|| self.workspace.default_view())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, kind: AssetKind) -> Asset {
        Asset {
            path: PathBuf::from(format!("/t/{name}")),
            name: name.into(),
            ext: name.rsplit('.').next().unwrap_or("").into(),
            kind,
            size: 1,
            modified: None,
            created: None,
            hidden: name.starts_with('.'),
        }
    }

    fn browser(names: &[&str]) -> Browser {
        let mut b = Browser::new(PathBuf::from("/t"));
        b.assets = names.iter().map(|n| asset(n, if n.contains('.') { AssetKind::Image } else { AssetKind::Folder })).collect();
        b.refresh_visible();
        b
    }

    fn names(b: &Browser) -> Vec<&str> {
        b.visible_assets().map(|a| a.name.as_str()).collect()
    }

    #[test]
    fn natural_order_puts_folders_first_and_hides_dotfiles() {
        let b = browser(&["IMG_10.jpg", "IMG_2.jpg", "sub", ".DS_Store"]);
        assert_eq!(names(&b), vec!["sub", "IMG_2.jpg", "IMG_10.jpg"]);
    }

    #[test]
    fn shift_click_selects_the_range_from_the_anchor() {
        let mut b = browser(&["a.jpg", "b.jpg", "c.jpg", "d.jpg"]);
        b.click_select(Path::new("/t/b.jpg"), false, false);
        b.click_select(Path::new("/t/d.jpg"), false, true);
        let mut sel: Vec<_> = b.selection.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        sel.sort();
        assert_eq!(sel, vec!["/t/b.jpg", "/t/c.jpg", "/t/d.jpg"]);
    }

    #[test]
    fn rating_filter_and_label_toggle() {
        let mut b = browser(&["a.jpg", "b.jpg"]);
        b.click_select(Path::new("/t/a.jpg"), false, false);
        b.set_rating(4);
        b.toggle_label(Some(Label::Select));
        b.quick = QuickFilter::AtLeast(3);
        b.refresh_visible();
        assert_eq!(names(&b), vec!["a.jpg"]);
        b.toggle_label(Some(Label::Select));
        assert_eq!(b.label(Path::new("/t/a.jpg")), None);
    }

    #[test]
    fn range_with_a_vanished_anchor_does_nothing() {
        let mut b = browser(&["a.jpg"]);
        b.anchor = Some(PathBuf::from("/t/gone.jpg"));
        b.click_select(Path::new("/t/a.jpg"), false, true);
        assert!(b.selection.is_empty());
    }
}
