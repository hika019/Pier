//! Interface icons, drawn from simple shapes (no Adobe artwork: AGENTS.md clean-room rule).

use std::f32::consts::{PI, TAU};

use egui::epaint::{PathShape, PathStroke};
use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2};

use crate::theme::Tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Back,
    Forward,
    Caret,
    Clock,
    Camera,
    Refine,
    Aperture,
    RotateCcw,
    RotateCw,
    Search,
    Picture,
    Star,
    SortAsc,
    SortDesc,
    NewFolder,
    Trash,
    Lock,
    ViewThumbs,
    ViewDetails,
    ViewList,
    Menu,
    Pin,
    Clear,
    Plus,
    PlusSub,
    Folder,
    Computer,
    Home,
    Desktop,
    Document,
    Download,
    Collection,
    SmartCollection,
    SmallSquare,
    LargeSquare,
    TriangleRight,
    TriangleDown,
    Check,
    Play,
    Music,
}

fn line(p: &Painter, a: Pos2, b: Pos2, s: Stroke) {
    p.line_segment([a, b], s);
}

fn polyline(p: &Painter, pts: Vec<Pos2>, s: Stroke) {
    p.add(PathShape::line(pts, PathStroke::new(s.width, s.color)));
}

fn arc(c: Pos2, r: f32, from: f32, to: f32) -> Vec<Pos2> {
    const STEPS: usize = 20;
    (0..=STEPS).map(|i| from + (to - from) * i as f32 / STEPS as f32).map(|a| c + vec2(a.cos(), a.sin()) * r).collect()
}

/// A five-pointed star as convex pieces (a pentagon and five tips), since egui fills only convex polygons.
pub fn star(p: &Painter, c: Pos2, r: f32, color: Color32) {
    let outer = |i: usize| c + Vec2::angled(-PI / 2.0 + i as f32 * TAU / 5.0) * r;
    let inner = |i: usize| c + Vec2::angled(-PI / 2.0 + (i as f32 + 0.5) * TAU / 5.0) * r * 0.42;
    p.add(Shape::convex_polygon((0..5).map(inner).collect(), color, Stroke::NONE));
    for i in 0..5 {
        p.add(Shape::convex_polygon(vec![outer(i), inner(i), inner((i + 4) % 5)], color, Stroke::NONE));
    }
}

fn folder_shape(p: &Painter, r: Rect, color: Color32, fill: bool) {
    let tab = Rect::from_min_size(r.min, vec2(r.width() * 0.42, r.height() * 0.22));
    let body = Rect::from_min_max(pos2(r.min.x, r.min.y + r.height() * 0.16), r.max);
    if fill {
        p.rect_filled(tab, 1.0, color);
        p.rect_filled(body, 1.5, color);
    } else {
        let s = Stroke::new(1.2, color);
        p.rect_stroke(body, 1.5, s, egui::StrokeKind::Inside);
        p.rect_stroke(tab, 1.0, s, egui::StrokeKind::Inside);
    }
}

pub fn paint(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.5;
    let s = Stroke::new((r * 0.14).clamp(1.0, 1.8), color);
    let at = |x: f32, y: f32| c + vec2(x, y) * r;
    match icon {
        Icon::Back => polyline(p, vec![at(0.25, -0.55), at(-0.3, 0.0), at(0.25, 0.55)], Stroke::new(s.width * 1.3, color)),
        Icon::Forward => polyline(p, vec![at(-0.25, -0.55), at(0.3, 0.0), at(-0.25, 0.55)], Stroke::new(s.width * 1.3, color)),
        Icon::Caret => {
            p.add(Shape::convex_polygon(vec![at(-0.45, -0.2), at(0.45, -0.2), at(0.0, 0.3)], color, Stroke::NONE));
        }
        Icon::TriangleRight => {
            p.add(Shape::convex_polygon(vec![at(-0.3, -0.45), at(0.4, 0.0), at(-0.3, 0.45)], color, Stroke::NONE));
        }
        Icon::TriangleDown => {
            p.add(Shape::convex_polygon(vec![at(-0.45, -0.3), at(0.45, -0.3), at(0.0, 0.4)], color, Stroke::NONE));
        }
        Icon::Clock => {
            p.circle_stroke(c, r * 0.75, s);
            polyline(p, vec![at(0.0, -0.45), at(0.0, 0.0), at(0.35, 0.2)], s);
        }
        Icon::Camera => {
            let body = Rect::from_center_size(at(0.0, 0.1), vec2(r * 1.7, r * 1.15));
            p.rect_stroke(body, 2.0, s, egui::StrokeKind::Inside);
            p.rect_filled(Rect::from_center_size(at(-0.25, -0.55), vec2(r * 0.5, r * 0.25)), 1.0, color);
            p.circle_stroke(at(0.0, 0.12), r * 0.35, s);
        }
        Icon::Refine => {
            for (dx, dy) in [(-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)] {
                p.rect_stroke(Rect::from_center_size(at(dx, dy), Vec2::splat(r * 0.6)), 1.0, s, egui::StrokeKind::Inside);
            }
        }
        Icon::Aperture => {
            p.circle_stroke(c, r * 0.8, s);
            for i in 0..6 {
                let a = i as f32 * TAU / 6.0;
                line(p, c + Vec2::angled(a) * r * 0.8, c + Vec2::angled(a + 2.1) * r * 0.32, s);
            }
        }
        Icon::RotateCcw | Icon::RotateCw => {
            let ccw = icon == Icon::RotateCcw;
            let (from, to) = if ccw { (-0.2 * PI, 1.3 * PI) } else { (1.2 * PI, -0.3 * PI) };
            let pts = arc(c, r * 0.62, from, to);
            let tip = pts.last().copied().unwrap_or(c);
            polyline(p, pts, s);
            let dir = vec2(0.0, -1.0);
            let side = if ccw { vec2(-1.0, 0.0) } else { vec2(1.0, 0.0) };
            p.add(Shape::convex_polygon(vec![tip + dir * r * 0.35, tip + side * r * 0.4, tip - dir * r * 0.35], color, Stroke::NONE));
        }
        Icon::Search => {
            p.circle_stroke(at(-0.15, -0.15), r * 0.5, s);
            line(p, at(0.22, 0.22), at(0.65, 0.65), Stroke::new(s.width * 1.4, color));
        }
        Icon::Picture => {
            let fr = Rect::from_center_size(c, vec2(r * 1.7, r * 1.3));
            p.rect_stroke(fr, 1.5, s, egui::StrokeKind::Inside);
            p.add(Shape::convex_polygon(vec![at(-0.65, 0.5), at(-0.15, -0.1), at(0.25, 0.5)], color, Stroke::NONE));
            p.circle_filled(at(0.35, -0.2), r * 0.16, color);
        }
        Icon::Star => star(p, c, r * 0.85, color),
        Icon::SortAsc | Icon::SortDesc => {
            let up = icon == Icon::SortAsc;
            line(p, at(-0.3, -0.6), at(-0.3, 0.6), s);
            let (y0, y1) = if up { (-0.6, -0.15) } else { (0.6, 0.15) };
            polyline(p, vec![at(-0.6, y1), at(-0.3, y0), at(0.0, y1)], s);
            for (i, w) in [0.25f32, 0.45, 0.65].iter().enumerate() {
                let y = -0.45 + i as f32 * 0.45;
                let w = if up { *w } else { 0.9 - *w };
                line(p, at(0.15, y), at(0.15 + w, y), s);
            }
        }
        Icon::NewFolder => {
            folder_shape(p, Rect::from_center_size(at(-0.1, 0.1), vec2(r * 1.5, r * 1.15)), color, false);
            line(p, at(0.55, -0.75), at(0.55, -0.15), s);
            line(p, at(0.25, -0.45), at(0.85, -0.45), s);
        }
        Icon::Trash => {
            line(p, at(-0.6, -0.5), at(0.6, -0.5), s);
            line(p, at(-0.2, -0.7), at(0.2, -0.7), s);
            polyline(p, vec![at(-0.45, -0.5), at(-0.35, 0.7), at(0.35, 0.7), at(0.45, -0.5)], s);
            line(p, at(0.0, -0.25), at(0.0, 0.45), s);
        }
        Icon::Lock => {
            p.rect_filled(Rect::from_center_size(at(0.0, 0.3), vec2(r * 1.2, r * 0.85)), 1.5, color);
            polyline(p, arc(at(0.0, -0.15), r * 0.38, PI, TAU), s);
        }
        Icon::ViewThumbs => {
            for (dx, dy) in [(-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)] {
                p.rect_filled(Rect::from_center_size(at(dx, dy), Vec2::splat(r * 0.6)), 0.5, color);
            }
        }
        Icon::ViewDetails => {
            for dy in [-0.45, 0.45] {
                p.rect_filled(Rect::from_center_size(at(-0.45, dy), Vec2::splat(r * 0.55)), 0.5, color);
                line(p, at(-0.05, dy - 0.12), at(0.75, dy - 0.12), s);
                line(p, at(-0.05, dy + 0.15), at(0.5, dy + 0.15), s);
            }
        }
        Icon::ViewList | Icon::Menu => {
            for dy in [-0.5, 0.0, 0.5] {
                line(p, at(-0.65, dy), at(0.65, dy), s);
            }
        }
        Icon::Pin => {
            p.circle_filled(at(0.0, -0.3), r * 0.35, color);
            line(p, at(0.0, -0.1), at(0.0, 0.7), s);
        }
        Icon::Clear => {
            p.circle_stroke(c, r * 0.65, s);
            line(p, at(-0.45, 0.45), at(0.45, -0.45), s);
        }
        Icon::Plus | Icon::PlusSub => {
            let o = if icon == Icon::PlusSub { 0.2 } else { 0.0 };
            line(p, at(o, -0.55), at(o, 0.55), s);
            line(p, at(o - 0.55, 0.0), at(o + 0.55, 0.0), s);
            if icon == Icon::PlusSub {
                polyline(p, vec![at(-0.7, -0.6), at(-0.7, 0.6), at(-0.45, 0.6)], s);
            }
        }
        Icon::Folder => folder_shape(p, Rect::from_center_size(c, vec2(r * 1.7, r * 1.35)), color, true),
        Icon::Computer => {
            p.rect_stroke(Rect::from_center_size(at(0.0, -0.15), vec2(r * 1.6, r * 1.05)), 1.0, s, egui::StrokeKind::Inside);
            line(p, at(-0.4, 0.7), at(0.4, 0.7), s);
            line(p, at(0.0, 0.4), at(0.0, 0.7), s);
        }
        Icon::Home => {
            polyline(p, vec![at(-0.75, 0.0), at(0.0, -0.7), at(0.75, 0.0)], s);
            polyline(p, vec![at(-0.5, -0.15), at(-0.5, 0.7), at(0.5, 0.7), at(0.5, -0.15)], s);
        }
        Icon::Desktop => {
            p.rect_stroke(Rect::from_center_size(c, vec2(r * 1.6, r * 1.3)), 1.0, s, egui::StrokeKind::Inside);
            line(p, at(-0.8, -0.35), at(0.8, -0.35), s);
        }
        Icon::Document => {
            polyline(
                p,
                vec![at(0.2, -0.75), at(-0.55, -0.75), at(-0.55, 0.75), at(0.55, 0.75), at(0.55, -0.4), at(0.2, -0.75), at(0.2, -0.4), at(0.55, -0.4)],
                s,
            );
        }
        Icon::Download => {
            line(p, at(0.0, -0.7), at(0.0, 0.3), s);
            polyline(p, vec![at(-0.4, -0.05), at(0.0, 0.35), at(0.4, -0.05)], s);
            line(p, at(-0.65, 0.7), at(0.65, 0.7), s);
        }
        Icon::Collection | Icon::SmartCollection => {
            for i in 0..3 {
                let o = i as f32 * 0.22;
                p.rect_stroke(Rect::from_center_size(at(-0.2 + o, -0.2 + o), Vec2::splat(r * 0.9)), 1.0, s, egui::StrokeKind::Inside);
            }
            if icon == Icon::SmartCollection {
                star(p, at(0.55, -0.55), r * 0.35, color);
            }
        }
        Icon::SmallSquare => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::splat(r * 0.75)), 0.5, s, egui::StrokeKind::Inside);
        }
        Icon::LargeSquare => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::splat(r * 1.45)), 0.5, s, egui::StrokeKind::Inside);
        }
        Icon::Check => polyline(p, vec![at(-0.55, 0.0), at(-0.15, 0.45), at(0.6, -0.5)], Stroke::new(s.width * 1.3, color)),
        Icon::Play => {
            p.add(Shape::convex_polygon(vec![at(-0.4, -0.6), at(0.6, 0.0), at(-0.4, 0.6)], color, Stroke::NONE));
        }
        Icon::Music => {
            p.circle_filled(at(-0.35, 0.45), r * 0.25, color);
            p.circle_filled(at(0.45, 0.3), r * 0.25, color);
            polyline(p, vec![at(-0.12, 0.45), at(-0.12, -0.55), at(0.68, -0.7), at(0.68, 0.3)], s);
        }
    }
}

/// A frameless icon button that highlights on hover, like Bridge's application-bar buttons.
pub fn button(ui: &mut Ui, t: &Tokens, icon: Icon, size: f32, tip: &str, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), if enabled { Sense::click() } else { Sense::hover() });
    if enabled && resp.hovered() {
        ui.painter().rect_filled(rect, t.radius_small, t.bg_cell_hover);
    }
    let color = if enabled { t.fg_icon } else { t.fg_disabled };
    paint(ui.painter(), rect.shrink(size * 0.2), icon, color);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, tip));
    resp.on_hover_text(tip)
}

/// An icon button with a small caret, that opens a menu.
pub fn menu_button(ui: &mut Ui, t: &Tokens, icon: Icon, size: f32, tip: &str, add: impl FnOnce(&mut Ui)) {
    popup_button(ui, t, icon, size, tip, true, add);
}

/// Like [`menu_button`]; `caret: false` draws the icon alone (the path bar's ▸ separators).
pub fn popup_button(ui: &mut Ui, t: &Tokens, icon: Icon, size: f32, tip: &str, caret: bool, add: impl FnOnce(&mut Ui)) {
    let width = if caret { size + 9.0 } else { size };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, size), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, t.radius_small, t.bg_cell_hover);
    }
    let icon_rect = Rect::from_min_size(rect.min, Vec2::splat(size));
    paint(ui.painter(), icon_rect.shrink(size * 0.2), icon, t.fg_icon);
    if caret {
        paint(ui.painter(), Rect::from_center_size(pos2(rect.right() - 6.0, rect.center().y), Vec2::splat(7.0)), Icon::Caret, t.fg_icon);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tip));
    let resp = resp.on_hover_text(tip);
    egui::Popup::menu(&resp).show(add);
}
