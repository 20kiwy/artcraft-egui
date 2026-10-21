//! A small hand-painted icon set (thin 1.6 px strokes, like the webapp's icon
//! language). No icon fonts, no external assets — pure epaint.

use egui::{Color32, CornerRadius, Rect, Shape, Stroke, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Icon {
  Home,
  Image,
  Chat,
  Library,
  Settings,
  Wand,
  Send,
  Download,
  Plus,
  Close,
  Minimize,
  Maximize,
  Refresh,
  Trash,
  Folder,
  Plug,
}

/// Paints `icon` centered in `rect` with `color`.
pub fn paint(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
  let st = Stroke::new(1.6, color);
  let Rect { min, max } = rect;
  let cx = rect.center().x;
  let cy = rect.center().y;
  match icon {
    Icon::Home => {
      p.add(Shape::line(vec![pos2(min.x, cy), pos2(cx, min.y), pos2(max.x, cy)], st));
      p.add(Shape::line(vec![pos2(min.x + 2.5, cy), pos2(min.x + 2.5, max.y - 1.0), pos2(max.x - 2.5, max.y - 1.0), pos2(max.x - 2.5, cy)], st));
    },
    Icon::Image => {
      p.rect(Rect::from_two_pos(min, max), CornerRadius::same(2), Color32::TRANSPARENT, st, egui::StrokeKind::Inside);
      p.add(Shape::line(vec![pos2(min.x + 2.0, max.y - 3.0), pos2(min.x + 7.0, cy + 1.0), pos2(cx + 1.0, max.y - 3.0)], st));
      p.circle_filled(pos2(max.x - 5.5, min.y + 5.5), 1.8, color);
    },
    Icon::Chat => {
      p.rect(Rect::from_two_pos(min, pos2(max.x, max.y - 4.0)), CornerRadius::same(3), Color32::TRANSPARENT, st, egui::StrokeKind::Inside);
      p.add(Shape::line(vec![pos2(cx - 3.0, max.y - 3.0), pos2(cx - 3.0, max.y), pos2(cx + 2.0, max.y - 4.5)], st));
      p.add(Shape::line(vec![pos2(min.x + 4.0, cy - 2.5), pos2(max.x - 4.0, cy - 2.5)], st));
      p.add(Shape::line(vec![pos2(min.x + 4.0, cy + 2.0), pos2(max.x - 7.0, cy + 2.0)], st));
    },
    Icon::Library => {
      p.add(Shape::line(vec![pos2(min.x + 2.0, min.y + 2.0), pos2(min.x + 2.0, max.y - 2.0)], st));
      for i in 0..3 {
        let y = min.y + 4.0 + i as f32 * ((rect.height() - 8.0) / 2.0);
        p.add(Shape::line(vec![pos2(min.x + 6.5, y), pos2(max.x - 2.0, y)], st));
      }
    },
    Icon::Settings => {
      p.circle_stroke(pos2(cx, cy), rect.width() * 0.26, st);
      for i in 0..6 {
        let ang = i as f32 * std::f32::consts::TAU / 6.0 + 0.35;
        let (dx, dy) = (ang.sin(), -ang.cos());
        p.add(Shape::line(vec![pos2(cx + dx * rect.width() * 0.34, cy + dy * rect.width() * 0.34), pos2(cx + dx * rect.width() * 0.46, cy + dy * rect.width() * 0.46)], st));
      }
    },
    Icon::Wand => {
      p.add(Shape::line(vec![pos2(min.x + 2.0, max.y - 2.0), pos2(cx + 1.5, cy - 1.5)], st));
      for (sx, sy) in [(max.x - 3.5, min.y + 3.5), (max.x - 8.5, min.y + 8.5)] {
        p.add(Shape::line(vec![pos2(sx - 2.5, sy), pos2(sx + 2.5, sy)], st));
        p.add(Shape::line(vec![pos2(sx, sy - 2.5), pos2(sx, sy + 2.5)], st));
      }
    },
    Icon::Send => {
      p.add(Shape::convex_polygon(vec![pos2(min.x, min.y + 1.0), pos2(max.x, cy), pos2(min.x, max.y - 1.0)], color, Stroke::NONE));
    },
    Icon::Download => {
      p.add(Shape::line(vec![pos2(cx, min.y + 1.5), pos2(cx, max.y - 5.0)], st));
      p.add(Shape::line(vec![pos2(cx - 4.0, max.y - 8.0), pos2(cx, max.y - 4.5), pos2(cx + 4.0, max.y - 8.0)], st));
      p.add(Shape::line(vec![pos2(min.x + 1.0, max.y - 1.5), pos2(max.x - 1.0, max.y - 1.5)], st));
    },
    Icon::Plus => {
      p.add(Shape::line(vec![pos2(cx, min.y + 1.5), pos2(cx, max.y - 1.5)], st));
      p.add(Shape::line(vec![pos2(min.x + 1.5, cy), pos2(max.x - 1.5, cy)], st));
    },
    Icon::Close => {
      p.add(Shape::line(vec![pos2(min.x + 2.0, min.y + 2.0), pos2(max.x - 2.0, max.y - 2.0)], st));
      p.add(Shape::line(vec![pos2(max.x - 2.0, min.y + 2.0), pos2(min.x + 2.0, max.y - 2.0)], st));
    },
    Icon::Minimize => {
      p.add(Shape::line(vec![pos2(min.x + 2.0, cy), pos2(max.x - 2.0, cy)], st));
    },
    Icon::Maximize => {
      p.rect(Rect::from_two_pos(pos2(min.x + 2.5, min.y + 2.5), pos2(max.x - 2.5, max.y - 2.5)), CornerRadius::same(1), Color32::TRANSPARENT, st, egui::StrokeKind::Inside);
    },
    Icon::Refresh => {
      p.circle_stroke(pos2(cx, cy), rect.width() * 0.34, st);
      p.add(Shape::convex_polygon(vec![pos2(max.x - 1.0, min.y + 3.0), pos2(max.x - 1.0, min.y + 9.5), pos2(cx + 1.5, min.y + 6.0)], color, Stroke::NONE));
    },
    Icon::Trash => {
      p.rect(Rect::from_two_pos(pos2(min.x + 3.5, min.y + 4.5), pos2(max.x - 3.5, max.y - 1.5)), CornerRadius::same(1), Color32::TRANSPARENT, st, egui::StrokeKind::Inside);
      p.add(Shape::line(vec![pos2(min.x + 1.5, min.y + 4.5), pos2(max.x - 1.5, min.y + 4.5)], st));
      p.add(Shape::line(vec![pos2(cx - 3.0, min.y + 4.5), pos2(cx - 2.0, min.y + 1.5), pos2(cx + 2.0, min.y + 1.5), pos2(cx + 3.0, min.y + 4.5)], st));
    },
    Icon::Folder => {
      p.add(Shape::line(vec![pos2(min.x + 1.5, max.y - 2.0), pos2(min.x + 1.5, min.y + 3.5), pos2(cx - 2.0, min.y + 3.5), pos2(cx + 1.0, min.y + 6.5), pos2(max.x - 1.5, min.y + 6.5), pos2(max.x - 1.5, max.y - 2.0), pos2(min.x + 1.5, max.y - 2.0)], st));
    },
    Icon::Plug => {
      p.circle_stroke(pos2(cx - 3.0, cy), rect.width() * 0.22, st);
      p.add(Shape::line(vec![pos2(cx - 1.0, cy), pos2(max.x - 2.0, cy)], st));
      p.add(Shape::line(vec![pos2(max.x - 4.0, cy - 2.5), pos2(max.x - 1.0, cy - 2.5)], st));
      p.add(Shape::line(vec![pos2(max.x - 4.0, cy + 2.5), pos2(max.x - 1.0, cy + 2.5)], st));
    },
  }
}

/// Fit `size` into `avail` without upscaling.
pub fn fit(size: egui::Vec2, avail: egui::Vec2) -> egui::Vec2 {
  if size.x <= 0.0 || size.y <= 0.0 {
    return vec2(avail.x.min(320.0), avail.x.min(320.0));
  }
  let scale = (avail.x / size.x).min(avail.y / size.y).min(1.0);
  size * scale
}
