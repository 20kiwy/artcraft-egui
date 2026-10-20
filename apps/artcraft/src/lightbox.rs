//! Full-screen image overlay: dimmed backdrop, fitted image, caption and a
//! close/reveal row. Click anywhere to dismiss.

use egui::{Align2, Color32, FontId, Id, Order, Rect, Sense, pos2};

use crate::app::ArtcraftApp;
use crate::icons;
use crate::theme;

pub fn show(app: &mut ArtcraftApp, ctx: &egui::Context) {
  if app.lightbox.is_none() {
    return;
  }
  let mut close = false;
  let mut reveal: Option<std::path::PathBuf> = None;

  egui::Area::new(Id::new("lightbox"))
    .order(Order::Foreground)
    .fixed_pos(pos2(0.0, 0.0))
    .show(ctx, |ui| {
      let screen = ui.max_rect().size();
      let (rect, resp) = ui.allocate_exact_size(screen, Sense::click());
      ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(226));

      if let Some(lb) = &mut app.lightbox {
        let avail = rect.shrink2(egui::vec2(48.0, 110.0));
        let fit = icons::fit(lb.tex.size_vec2(), avail.size());
        let image_rect = Rect::from_center_size(rect.center() - egui::vec2(0.0, 14.0), fit);
        egui::Image::new(&lb.tex).corner_radius(theme::RADIUS).paint_at(ui, image_rect);

        // Caption under the image.
        ui.painter().text(
          pos2(rect.center().x, image_rect.bottom() + 26.0),
          Align2::CENTER_TOP,
          &lb.caption,
          FontId::new(13.0, theme::mono()),
          theme::MUTED,
        );

        // Close button, top-right.
        let close_rect = Rect::from_min_size(pos2(rect.right() - 52.0, rect.top() + 16.0), egui::vec2(36.0, 36.0));
        let (_, btn_resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::click());
        let fill = if btn_resp.hovered() { Color32::from_white_alpha(38) } else { Color32::TRANSPARENT };
        ui.painter().rect_filled(close_rect, theme::RADIUS, fill);
        icons::paint(ui.painter(), Rect::from_center_size(close_rect.center(), egui::vec2(14.0, 14.0)), icons::Icon::Close, Color32::WHITE);
        if btn_resp.clicked() {
          close = true;
        }
        btn_resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close lightbox"));

        // Reveal in explorer, bottom-right.
        if lb.file.is_some() {
          let r = Rect::from_min_size(pos2(rect.right() - 190.0, rect.bottom() - 52.0), egui::vec2(170.0, 32.0));
          let rresp = ui.allocate_rect(r, Sense::click());
          let fill = if rresp.hovered() { theme::CONTROLS } else { theme::PANEL };
          ui.painter().rect(r, theme::RADIUS, fill, theme::hairline(), egui::StrokeKind::Inside);
          icons::paint(ui.painter(), Rect::from_center_size(pos2(r.left() + 22.0, r.center().y), egui::vec2(14.0, 14.0)), icons::Icon::Folder, theme::INK);
          ui.painter().text(pos2(r.left() + 38.0, r.center().y), Align2::LEFT_CENTER, "Reveal in Explorer", FontId::new(12.5, theme::medium()), theme::INK);
          if rresp.clicked() {
            reveal = lb.file.clone();
          }
          rresp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Reveal in Explorer"));
        }
      }

      if resp.clicked() {
        close = true;
      }
    });

  if close {
    app.lightbox = None;
  }
  if let Some(path) = reveal {
    let _ = open::that_detached(path.parent().unwrap_or(std::path::Path::new(".")));
  }
}
