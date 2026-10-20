//! The custom 40 px title bar: mark, proxy status chip (click → Settings) and
//! Windows-style caption buttons. The window is created with
//! `with_decorations(false)` like the upstream native client.

use egui::{Align, Color32, CornerRadius, Layout, Rect, Sense, Stroke, StrokeKind, Ui, ViewportCommand, pos2, vec2};

use crate::app::ConnState;
use crate::theme;
use crate::widgets;

/// Renders the title bar; returns `true` when the status chip was clicked.
pub fn render(ui: &mut Ui, conn: &ConnState, proxy_url: &str) -> bool {
  let mut open_settings = false;

  egui::Panel::top("titlebar")
    .exact_size(crate::shell::TITLEBAR_HEIGHT)
    .resizable(false)
    .frame(egui::Frame::NONE.fill(theme::CHROME))
    .show(ui, |ui| {
      // Window drag + double-click maximize over the whole bar. Registered
      // first so the widgets painted after it win the interactions.
      let full = ui.max_rect();
      let drag = ui.interact(full, egui::Id::new("titlebar-drag"), Sense::click_and_drag());
      if drag.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
      }
      if drag.double_clicked() {
        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
      }

      ui.horizontal_centered(|ui| {
        // The mark: a small accent tile.
        let (mark, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        ui.painter().rect_filled(mark, CornerRadius::same(2), theme::ACCENT);
        widgets::hud(ui, "artcraft", theme::INK);
        widgets::badge(ui, "egui", theme::BADGE_BETA);

        ui.add_space(10.0);

        // Proxy status chip → opens Settings.
        let (text, dot_color, tip) = match conn {
          ConnState::Checking => ("proxy · checking".to_owned(), theme::DIM, format!("Connecting to {proxy_url}…")),
          ConnState::Online { mode } => (format!("proxy · {mode}"), live_dot(mode), format!("ZAI plugin online at {proxy_url} — click for settings")),
          ConnState::Offline { err } => ("proxy · offline".to_owned(), theme::BAD, format!("{proxy_url} unreachable: {err} — click for settings")),
        };
        let galley = widgets::caps(ui, &text, theme::HUD_SIZE, theme::mono(), theme::HUD_SPACING, theme::MUTED);
        let chip_size = vec2(galley.size().x + 34.0, 24.0);
        let (chip_rect, chip) = ui.allocate_exact_size(chip_size, Sense::click());
        ui.painter().rect(chip_rect, theme::RADIUS, theme::CONTROLS, Stroke::new(1.0, if chip.hovered() { theme::LINE_STRONG } else { theme::LINE }), StrokeKind::Inside);
        widgets::dot(ui.painter(), pos2(chip_rect.left() + 14.0, chip_rect.center().y), dot_color);
        ui.painter().galley(pos2(chip_rect.left() + 24.0, chip_rect.center().y - galley.size().y / 2.0), galley, theme::MUTED);
        let chip = chip.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand);
        if chip.clicked() {
          open_settings = true;
        }

        // Caption buttons, right-aligned.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
          caption_button(ui, "close", |ui| {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
          });
          caption_button(ui, "maximize", |ui| {
            let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
          });
          caption_button(ui, "minimize", |ui| {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
          });
        });
      });
    });

  open_settings
}

fn live_dot(mode: &str) -> Color32 {
  if mode == "live" {
    theme::OK
  } else {
    theme::WARN
  }
}

/// A 44 px-wide caption button with a hand-painted glyph; close hovers red.
fn caption_button(ui: &mut Ui, tip: &str, action: impl FnOnce(&mut Ui)) {
  let (rect, resp) = ui.allocate_exact_size(vec2(44.0, ui.available_height()), Sense::click());
  let is_close = tip == "close";
  let (fill, ink) = if resp.hovered() {
    (if is_close { theme::DANGER } else { theme::WASH_HOVER }, if is_close { Color32::WHITE } else { theme::INK })
  } else {
    (Color32::TRANSPARENT, theme::DIM)
  };
  ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
  let glyph_rect = Rect::from_center_size(rect.center(), vec2(12.0, 12.0));
  let p = ui.painter();
  if is_close {
    p.line_segment([glyph_rect.left_top(), glyph_rect.right_bottom()], Stroke::new(1.4, ink));
    p.line_segment([glyph_rect.right_top(), glyph_rect.left_bottom()], Stroke::new(1.4, ink));
  } else if tip == "minimize" {
    p.line_segment([pos2(glyph_rect.left(), glyph_rect.center().y), pos2(glyph_rect.right(), glyph_rect.center().y)], Stroke::new(1.4, ink));
  } else {
    p.rect(Rect::from_min_size(glyph_rect.min, glyph_rect.size()), CornerRadius::same(1), Color32::TRANSPARENT, Stroke::new(1.4, ink), StrokeKind::Inside);
  }
  resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
  if resp.clicked() {
    action(ui);
  }
}
