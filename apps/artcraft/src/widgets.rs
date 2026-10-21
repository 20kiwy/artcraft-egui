//! Building blocks in the webapp's brutalist style: mono HUD labels, toolbar
//! pills, the inverted primary button, status dots and hairline separators.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontId, Frame, Galley, Margin, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::icons::{self, Icon};
use crate::theme;

/// `text` laid out upper case in `family`, `spacing` points between letters.
pub fn caps(ui: &Ui, text: &str, size: f32, family: egui::FontFamily, spacing: f32, color: Color32) -> Arc<Galley> {
  let format = egui::TextFormat { font_id: FontId::new(size, family), color, extra_letter_spacing: spacing, ..Default::default() };
  ui.painter().layout_job(egui::text::LayoutJob::single_section(text.to_uppercase(), format))
}

/// A `.hud-label`: 11 px Geist Mono, upper case, widely spaced.
pub fn hud(ui: &mut Ui, text: &str, color: Color32) -> Response {
  let galley = caps(ui, text, theme::HUD_SIZE, theme::mono(), theme::HUD_SPACING, color);
  let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
  ui.painter().galley(rect.min, galley, color);
  resp
}

/// A sidebar section heading: 11 px Geist Mono 600, upper case.
pub fn section_label(ui: &mut Ui, text: &str, color: Color32) {
  let galley = caps(ui, text, theme::HUD_SIZE, theme::mono_bold(), theme::LABEL_SPACING, color);
  let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
  ui.painter().galley(pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0), galley, color);
}

/// A small badge after a label ("EGUI", "BETA"): 9 px mono on a tinted chip.
pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
  let galley = caps(ui, text, 9.0, theme::mono_bold(), 0.9, color);
  let size = galley.size() + vec2(10.0, 4.0);
  let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
  ui.painter().rect(rect, CornerRadius::same(2), theme::fade(color, 0.12), Stroke::new(1.0, theme::fade(color, 0.4)), StrokeKind::Inside);
  ui.painter().galley(rect.center() - galley.size() / 2.0, galley, color);
}

/// Midpoint blend of two colours (used for hover states).
pub fn blend(a: Color32, b: Color32) -> Color32 {
  let f = |x: u8, y: u8| ((x as u16 + y as u16) / 2) as u8;
  Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// A tiny status dot.
pub fn dot(p: &egui::Painter, center: egui::Pos2, color: Color32) {
  p.circle_filled(center, 3.5, color);
}

/// A 1 px hairline across the full width.
pub fn hairline(ui: &mut Ui) {
  let h = ui.available_height();
  let w = ui.available_width();
  let (rect, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
  ui.painter().line_segment([rect.left(), rect.right()].map(|x| pos2(x, rect.center().y)), theme::hairline());
  let _ = h;
}

/// A transparent toolbar button with an icon and an optional trailing label.
pub fn ghost(ui: &mut Ui, icon: Icon, label: Option<&str>, tip: &str, active: bool) -> Response {
  let enabled = ui.is_enabled();
  let galley = label.map(|l| ui.painter().layout_no_wrap(l.to_owned(), FontId::new(14.0, theme::medium()), theme::INK));
  let w = match &galley {
    Some(g) => 10.0 + 15.0 + 6.0 + g.size().x + 10.0,
    None => 36.0,
  };
  let (rect, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
  let hovered = resp.hovered() && enabled;
  let p = ui.painter();
  if hovered || active {
    p.rect_filled(rect, theme::RADIUS, theme::WASH);
  }
  let ink = match (enabled, hovered || active) {
    (false, _) => theme::fade(theme::DIM, 0.5),
    (true, true) => theme::INK,
    (true, false) => theme::DIM,
  };
  match galley {
    Some(g) => {
      let icon_rect = Rect::from_center_size(pos2(rect.left() + 10.0 + 7.5, rect.center().y), vec2(15.0, 15.0));
      icons::paint(p, icon_rect, icon, ink);
      p.galley_with_override_text_color(pos2(icon_rect.right() + 6.0, rect.center().y - g.size().y / 2.0), g, ink);
    },
    None => icons::paint(p, Rect::from_center_size(rect.center(), vec2(16.0, 16.0)), icon, ink),
  }
  let resp = resp.on_hover_text(tip);
  if enabled {
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
  } else {
    resp
  }
}

/// The inverted primary button: white block, near-black text, 3 px corners.
pub fn primary(ui: &mut Ui, icon: Option<Icon>, label: &str) -> Response {
  let galley = ui.painter().layout_no_wrap(label.to_owned(), FontId::new(14.0, theme::semibold()), theme::INVERT_FG);
  let icon_w = if icon.is_some() { 21.0 } else { 0.0 };
  let w = 14.0 + icon_w + galley.size().x + 14.0;
  let (rect, resp) = ui.allocate_exact_size(vec2(w.max(theme::CONTROL_H + 6.0), 38.0), Sense::click());
  let p = ui.painter();
  let fill = if resp.hovered() { blend(theme::INVERT_BG, Color32::from_gray(215)) } else { theme::INVERT_BG };
  p.rect(rect, theme::RADIUS, fill, Stroke::NONE, StrokeKind::Inside);
  let mut x = rect.left() + 14.0;
  if let Some(icon) = icon {
    icons::paint(p, Rect::from_center_size(pos2(x + 7.5, rect.center().y), vec2(15.0, 15.0)), icon, theme::INVERT_FG);
    x += icon_w;
  }
  p.galley_with_override_text_color(pos2(x, rect.center().y - galley.size().y / 2.0), galley, theme::INVERT_FG);
  resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
  resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A well/panel card: `ui-panel` fill on a hairline, 8 px corners.
pub fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
  Frame::default().fill(theme::PANEL).corner_radius(CornerRadius::same(8)).stroke(theme::hairline()).inner_margin(Margin::same(14)).show(ui, |ui| add(ui));
}

/// The prompt-box container: `ui-controls` on a hairline, 3 px corners.
pub fn prompt_box(ui: &mut Ui, add: impl FnOnce(&mut Ui)) -> egui::InnerResponse<()> {
  Frame::default().fill(theme::CONTROLS).corner_radius(theme::RADIUS).stroke(theme::hairline()).inner_margin(Margin::same(12)).show(ui, add)
}

/// The multiline text editor docked composers can trust: the box starts at
/// `rows` lines and grows while typing, but never past `max_height` — beyond
/// that the text scrolls *inside* the box, so a long pasted prompt can never
/// stretch the docked composer over the page ("why is its height so big").
pub fn capped_multiline(ui: &mut Ui, text: &mut String, hint: &str, rows: usize, max_height: f32, font: FontId) -> Response {
  let mut inner: Option<Response> = None;
  egui::ScrollArea::vertical()
    .auto_shrink([false, true])
    .max_height(max_height)
    .show(ui, |ui| {
      let resp = egui::TextEdit::multiline(text)
        .hint_text(hint)
        .desired_rows(rows)
        .desired_width(ui.available_width())
        .frame(Frame::NONE)
        .font(font)
        .show(ui);
      // egui 0.36: TextEditOutput.response is an AtomLayoutResponse (Deref→Response).
      inner = Some(resp.response.response.clone());
    });
  inner.expect("scroll-area body runs exactly once")
}

/// Left-aligned mono caption + right-aligned mono value on one row.
pub fn kv(ui: &mut Ui, key: &str, value: &str, value_color: Color32) {
  ui.horizontal(|ui| {
    hud(ui, key, theme::FAINT);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
      hud(ui, value, value_color);
    });
  });
}
