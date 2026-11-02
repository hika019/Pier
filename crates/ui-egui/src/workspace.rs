//! Bridge's default workspaces and their panel layouts (docs/ui-design.md §5).

use crate::i18n::S;
use crate::state::ViewMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Workspace {
    Essentials,
    Filmstrip,
    Metadata,
    Keywords,
    Preview,
    LightTable,
    Folders,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    Favorites,
    Folders,
    Filter,
    Collections,
    Export,
    Content,
    Preview,
    Metadata,
    Keywords,
    Output,
    OutputPreview,
}

impl PanelId {
    pub fn id(self) -> &'static str {
        match self {
            Self::Favorites => "favorites",
            Self::Folders => "folders",
            Self::Filter => "filter",
            Self::Collections => "collections",
            Self::Export => "export",
            Self::Content => "content",
            Self::Preview => "preview",
            Self::Metadata => "metadata",
            Self::Keywords => "keywords",
            Self::Output => "output",
            Self::OutputPreview => "outputPreview",
        }
    }

    pub fn title(self) -> S {
        match self {
            Self::Favorites => S::PFavorites,
            Self::Folders => S::PFolders,
            Self::Filter => S::PFilter,
            Self::Collections => S::PCollections,
            Self::Export => S::PExport,
            Self::Content => S::PContent,
            Self::Preview => S::PPreview,
            Self::Metadata => S::PMetadata,
            Self::Keywords => S::PKeywords,
            Self::Output => S::POutput,
            Self::OutputPreview => S::POutputPreview,
        }
    }
}

/// A vertical stack of tabbed panel groups. `split` is the first group's share of the height.
pub struct Column {
    pub width: f32,
    pub groups: &'static [&'static [PanelId]],
    pub split: f32,
}

pub struct Layout {
    /// Outermost first.
    pub left: &'static [Column],
    pub center: Column,
    pub right: &'static [Column],
}

use PanelId as P;

const NAV: &[PanelId] = &[P::Favorites, P::Folders];
const ORGANIZE: &[PanelId] = &[P::Filter, P::Collections, P::Export];
const INFO: &[PanelId] = &[P::Metadata, P::Keywords];

const LEFT_ESSENTIALS: &[Column] = &[Column { width: 230.0, groups: &[NAV, ORGANIZE], split: 0.42 }];

impl Workspace {
    pub const ALL: [Self; 8] =
        [Self::Essentials, Self::Filmstrip, Self::Metadata, Self::Keywords, Self::Preview, Self::LightTable, Self::Folders, Self::Output];
    /// How many workspace names the application bar shows before the ▾ menu (Bridge shows as many as fit).
    pub const IN_BAR: usize = 4;

    pub fn name(self) -> S {
        match self {
            Self::Essentials => S::WsEssentials,
            Self::Filmstrip => S::WsFilmstrip,
            Self::Metadata => S::WsMetadata,
            Self::Keywords => S::WsKeywords,
            Self::Preview => S::WsPreview,
            Self::LightTable => S::WsLightTable,
            Self::Folders => S::WsFolders,
            Self::Output => S::WsOutput,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Essentials => "essentials",
            Self::Filmstrip => "filmstrip",
            Self::Metadata => "metadata",
            Self::Keywords => "keywords",
            Self::Preview => "preview",
            Self::LightTable => "lightTable",
            Self::Folders => "folders",
            Self::Output => "output",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.id() == id)
    }

    pub fn default_view(self) -> ViewMode {
        match self {
            Self::Metadata => ViewMode::List,
            Self::Keywords => ViewMode::Details,
            _ => ViewMode::Thumbnails,
        }
    }

    pub fn layout(self) -> Layout {
        match self {
            Self::Essentials => Layout {
                left: LEFT_ESSENTIALS,
                center: Column { width: 0.0, groups: &[&[P::Content]], split: 1.0 },
                right: &[Column { width: 300.0, groups: &[&[P::Preview], INFO], split: 0.42 }],
            },
            Self::Filmstrip => {
                Layout { left: LEFT_ESSENTIALS, center: Column { width: 0.0, groups: &[&[P::Preview], &[P::Content]], split: 0.74 }, right: &[] }
            }
            Self::Metadata => Layout {
                left: &[Column { width: 280.0, groups: &[NAV, INFO], split: 0.34 }],
                center: Column { width: 0.0, groups: &[&[P::Content]], split: 1.0 },
                right: &[],
            },
            Self::Keywords => Layout {
                left: &[Column { width: 260.0, groups: &[NAV, &[P::Keywords]], split: 0.4 }],
                center: Column { width: 0.0, groups: &[&[P::Content]], split: 1.0 },
                right: &[],
            },
            Self::Preview => Layout {
                left: &[Column { width: 210.0, groups: &[NAV, ORGANIZE], split: 0.42 }, Column { width: 200.0, groups: &[&[P::Content]], split: 1.0 }],
                center: Column { width: 0.0, groups: &[&[P::Preview], INFO], split: 0.7 },
                right: &[],
            },
            Self::LightTable => Layout { left: &[], center: Column { width: 0.0, groups: &[&[P::Content]], split: 1.0 }, right: &[] },
            Self::Folders => Layout {
                left: &[Column { width: 320.0, groups: &[&[P::Folders]], split: 1.0 }],
                center: Column { width: 0.0, groups: &[&[P::Content]], split: 1.0 },
                right: &[],
            },
            Self::Output => Layout {
                left: &[Column { width: 300.0, groups: &[&[P::Content]], split: 1.0 }],
                center: Column { width: 0.0, groups: &[&[P::OutputPreview]], split: 1.0 },
                right: &[Column { width: 320.0, groups: &[&[P::Output]], split: 1.0 }],
            },
        }
    }
}
