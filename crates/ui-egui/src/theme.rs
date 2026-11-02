//! Design tokens. Every colour, size and radius the shell draws comes from [`Tokens`].
//!
//! The values imitate Bridge's four interface brightness levels by eye; they are provisional
//! until measured (docs/ui-design.md §8, `要計測`).

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

/// Preferences › Interface › Color Theme: Bridge's four brightness steps, darkest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Brightness {
    #[default]
    Darkest,
    Dark,
    Light,
    Lightest,
}

impl Brightness {
    pub const ALL: [Self; 4] = [Self::Darkest, Self::Dark, Self::Light, Self::Lightest];

    pub fn name(self) -> crate::i18n::S {
        use crate::i18n::S;
        match self {
            Self::Darkest => S::BrightnessDarkest,
            Self::Dark => S::BrightnessDark,
            Self::Light => S::BrightnessLight,
            Self::Lightest => S::BrightnessLightest,
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Darkest | Self::Dark)
    }
}

/// Label colours, in Bridge's order (Select, Second, Approved, Review, To Do).
#[derive(Clone, Copy, Debug)]
pub struct LabelColors {
    pub select: Color32,
    pub second: Color32,
    pub approved: Color32,
    pub review: Color32,
    pub todo: Color32,
}

#[derive(Clone, Debug)]
pub struct Tokens {
    pub brightness: Brightness,
    /// Behind everything: the gaps between panels and the bars.
    pub bg_app: Color32,
    pub bg_bar: Color32,
    pub bg_panel: Color32,
    pub bg_tab_strip: Color32,
    pub bg_tab_active: Color32,
    pub bg_content: Color32,
    pub bg_cell_hover: Color32,
    pub bg_cell_selected: Color32,
    pub bg_field: Color32,
    pub bg_section: Color32,
    pub bg_placard: Color32,
    pub separator: Color32,
    pub fg_text: Color32,
    pub fg_dim: Color32,
    pub fg_disabled: Color32,
    pub fg_icon: Color32,
    pub fg_strong: Color32,
    pub accent: Color32,
    pub star: Color32,
    pub star_off: Color32,
    pub reject: Color32,
    pub labels: LabelColors,
    pub radius_small: CornerRadius,
    pub bar_height: f32,
    pub path_bar_height: f32,
    pub tab_height: f32,
    pub status_height: f32,
    pub row_height: f32,
    pub font_ui: f32,
    pub font_small: f32,
}

const LABELS: LabelColors = LabelColors {
    select: Color32::from_rgb(0xd8, 0x3b, 0x3b),
    second: Color32::from_rgb(0xe8, 0xc5, 0x2e),
    approved: Color32::from_rgb(0x4c, 0xb0, 0x4c),
    review: Color32::from_rgb(0x3b, 0x7f, 0xd8),
    todo: Color32::from_rgb(0x9b, 0x52, 0xc9),
};

const fn gray(v: u8) -> Color32 {
    Color32::from_rgb(v, v, v)
}

impl Tokens {
    pub fn for_brightness(brightness: Brightness) -> Self {
        // (app, bar, panel, tab strip, content, hover, selected, field, section, separator, text, dim, disabled, strong)
        let g = match brightness {
            Brightness::Darkest => [0x1a, 0x26, 0x2b, 0x20, 0x1e, 0x2e, 0x46, 0x1a, 0x33, 0x14, 0xd0, 0x8c, 0x5a, 0xf2],
            Brightness::Dark => [0x2c, 0x3a, 0x40, 0x33, 0x32, 0x46, 0x5c, 0x2a, 0x48, 0x24, 0xdc, 0x9c, 0x6c, 0xf8],
            Brightness::Light => [0x8c, 0xa8, 0xb4, 0x9c, 0xa6, 0xbc, 0xcc, 0xd8, 0xa8, 0x80, 0x18, 0x40, 0x78, 0x00],
            Brightness::Lightest => [0xc8, 0xe0, 0xec, 0xd4, 0xe4, 0xf2, 0xc4, 0xfa, 0xde, 0xb4, 0x1c, 0x5c, 0xa0, 0x00],
        };
        Self {
            brightness,
            bg_app: gray(g[0]),
            bg_bar: gray(g[1]),
            bg_panel: gray(g[2]),
            bg_tab_strip: gray(g[3]),
            bg_tab_active: gray(g[2]),
            bg_content: gray(g[4]),
            bg_cell_hover: gray(g[5]),
            bg_cell_selected: gray(g[6]),
            bg_field: gray(g[7]),
            bg_section: gray(g[8]),
            bg_placard: if brightness.is_dark() { gray(g[7]) } else { gray(0xf6) },
            separator: gray(g[9]),
            fg_text: gray(g[10]),
            fg_dim: gray(g[11]),
            fg_disabled: gray(g[12]),
            fg_icon: if brightness.is_dark() { gray(0xb8) } else { gray(0x3a) },
            fg_strong: gray(g[13]),
            accent: Color32::from_rgb(0x2f, 0x8c, 0xf0),
            star: if brightness.is_dark() { gray(0xe0) } else { gray(0x20) },
            star_off: if brightness.is_dark() { gray(0x60) } else { gray(0x90) },
            reject: Color32::from_rgb(0xe0, 0x40, 0x40),
            labels: LABELS,
            radius_small: CornerRadius::same(2),
            bar_height: 38.0,
            path_bar_height: 26.0,
            tab_height: 24.0,
            status_height: 26.0,
            row_height: 20.0,
            font_ui: 12.0,
            font_small: 10.5,
        }
    }

    pub fn font(&self) -> FontId {
        FontId::new(self.font_ui, FontFamily::Proportional)
    }

    pub fn font_small(&self) -> FontId {
        FontId::new(self.font_small, FontFamily::Proportional)
    }

    pub fn separator_stroke(&self) -> Stroke {
        Stroke::new(1.0, self.separator)
    }

    /// Applies the tokens to egui's own widgets (menus, text fields, sliders, scroll bars).
    pub fn apply(&self, ctx: &egui::Context) {
        let tokens = self.clone();
        ctx.all_styles_mut(move |style| {
            style.text_styles = [
                (TextStyle::Small, FontId::new(tokens.font_small, FontFamily::Proportional)),
                (TextStyle::Body, tokens.font()),
                (TextStyle::Button, tokens.font()),
                (TextStyle::Heading, FontId::new(tokens.font_ui + 2.0, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(tokens.font_ui, FontFamily::Monospace)),
            ]
            .into();
            style.spacing.item_spacing = egui::vec2(6.0, 3.0);
            style.spacing.button_padding = egui::vec2(6.0, 2.0);
            style.spacing.interact_size.y = tokens.row_height;
            style.spacing.menu_margin = egui::Margin::symmetric(4, 4);

            let mut v = if tokens.brightness.is_dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
            v.override_text_color = Some(tokens.fg_text);
            v.panel_fill = tokens.bg_panel;
            v.window_fill = tokens.bg_panel;
            v.extreme_bg_color = tokens.bg_field;
            v.faint_bg_color = tokens.bg_section;
            v.window_stroke = tokens.separator_stroke();
            v.menu_corner_radius = tokens.radius_small;
            v.window_corner_radius = CornerRadius::same(4);
            v.selection.bg_fill = tokens.accent;
            v.selection.stroke = Stroke::new(1.0, tokens.fg_strong);
            v.hyperlink_color = tokens.accent;
            v.widgets.noninteractive.bg_stroke = tokens.separator_stroke();
            v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.fg_text);
            for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
                w.corner_radius = tokens.radius_small;
            }
            v.widgets.inactive.weak_bg_fill = tokens.bg_field;
            v.widgets.inactive.bg_fill = tokens.bg_field;
            v.widgets.inactive.fg_stroke = Stroke::new(1.0, tokens.fg_text);
            v.widgets.hovered.weak_bg_fill = tokens.bg_cell_hover;
            v.widgets.hovered.bg_fill = tokens.bg_cell_hover;
            v.widgets.hovered.bg_stroke = Stroke::NONE;
            v.widgets.hovered.fg_stroke = Stroke::new(1.0, tokens.fg_strong);
            v.widgets.active.weak_bg_fill = tokens.bg_cell_selected;
            v.widgets.active.bg_fill = tokens.bg_cell_selected;
            v.widgets.open.weak_bg_fill = tokens.bg_cell_selected;
            style.visuals = v;
        });
    }
}
