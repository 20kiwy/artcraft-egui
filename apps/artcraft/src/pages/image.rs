//! Image studio: results feed + the docked prompt box (model, size, count,
//! Enter-to-generate), mirroring the upstream create-image page.

use egui::{CornerRadius, FontId, Sense, Stroke, StrokeKind, vec2};

use crate::app::ArtcraftApp;
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_image(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    // ---- results feed ----
    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("image-feed").show(ui, |ui| {
      if self.feed.is_empty() && !self.img_busy {
        ui.add_space(48.0);
        ui.vertical_centered(|ui| {
          ui.label(egui::RichText::new("Describe an image — ArtCraft will craft it.").size(16.0).color(theme::MUTED));
          ui.add_space(6.0);
          let mode = match &self.conn {
            crate::app::ConnState::Online { mode } => mode.clone(),
            _ => "…".to_owned(),
          };
          widgets::hud(ui, &format!("cogview via artcraft-proxy · mode: {mode}"), theme::FAINT);
        });
        ui.add_space(48.0);
      } else {
        let cols = ((ui.available_width() / 340.0).floor() as usize).clamp(1, 3);
        egui::Grid::new("feed-grid").spacing(vec2(10.0, 14.0)).show(ui, |ui| {
          for i in 0..self.feed.len() {
            self.tile(ui, i, ctx);
            if (i + 1) % cols == 0 || i + 1 == self.feed.len() {
              ui.end_row();
            }
          }
        });
      }
      if self.img_busy {
        ui.add_space(14.0);
        ui.horizontal_centered(|ui| {
          ui.add(egui::Spinner::new().size(18.0));
          ui.add_space(8.0);
          widgets::hud(ui, "crafting…", theme::ACCENT_INK);
        });
      }
      ui.add_space(12.0);
    });

    // ---- docked prompt box ----
    ui.add_space(4.0);
    widgets::prompt_box(ui, |ui| {
      let resp = egui::TextEdit::multiline(&mut self.img_prompt)
        .hint_text("Describe the image you want to craft…")
        .desired_rows(3)
        .desired_width(ui.available_width())
        .frame(egui::Frame::NONE)
        .font(FontId::new(15.0, theme::medium()))
        .show(ui);
      let enter = resp.response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !ui.input(|i| i.modifiers.shift);
      ui.add_space(6.0);
      ui.horizontal(|ui| {
        // Model picker.
        let models = self.cfg.as_ref().map(|c| c.image_models.clone()).unwrap_or_else(|| vec!["cogview-4".into(), "cogview-3-flash".into()]);
        let selected_model = self.img_model.clone().unwrap_or_else(|| models.first().cloned().unwrap_or_default());
        egui::ComboBox::from_id_salt("img-model")
          .selected_text(&selected_model)
          .width(170.0)
          .show_ui(ui, |ui| {
            for m in &models {
              ui.selectable_value(&mut self.img_model, Some(m.clone()), m);
            }
          });
        // Size picker.
        let sizes = self.cfg.as_ref().map(|c| c.image_sizes.clone()).unwrap_or_else(|| vec!["1024x1024".into(), "1152x896".into(), "1344x768".into(), "896x1152".into(), "768x1344".into()]);
        egui::ComboBox::from_id_salt("img-size")
          .selected_text(&self.img_size)
          .width(120.0)
          .show_ui(ui, |ui| {
            for s in &sizes {
              ui.selectable_value(&mut self.img_size, s.clone(), s);
            }
          });
        // Count picker.
        egui::ComboBox::from_id_salt("img-count")
          .selected_text(format!("{}×", self.img_count))
          .width(70.0)
          .show_ui(ui, |ui| {
            for n in 1..=4usize {
              ui.selectable_value(&mut self.img_count, n, format!("{n}×"));
            }
          });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          if self.img_busy {
            ui.add(egui::Spinner::new().size(16.0));
            ui.add_space(6.0);
            widgets::hud(ui, "crafting", theme::MUTED);
          } else {
            let gen = widgets::primary(ui, Some(Icon::Wand), "Generate");
            if gen.clicked() || enter {
              self.generate_images(ctx);
            }
          }
        });
      });
    });
  }

  fn tile(&mut self, ui: &mut egui::Ui, idx: usize, ctx: &egui::Context) {
    // Clone what we need up front so `self` can be borrowed mutably below.
    let Some((tex, prompt, model, size, file)) = self.feed.get(idx).map(|i| {
      (i.tex.clone(), i.prompt.clone(), i.model.clone(), i.size.clone(), i.file.clone())
    }) else {
      return;
    };

    let tile_w = ((ui.available_width() / 3.0) - 20.0).clamp(240.0, 360.0);
    let fit = icons::fit(tex.size_vec2(), vec2(tile_w, tile_w * 0.85));
    let (rect, resp) = ui.allocate_exact_size(fit, Sense::click());
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(8), theme::SUNKEN, Stroke::new(1.0, if resp.hovered() { theme::ACCENT_400 } else { theme::LINE }), StrokeKind::Inside);
    egui::Image::new(&tex).corner_radius(theme::RADIUS).paint_at(ui, rect.shrink(1.0));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, &prompt));
    if resp.hovered() {
      ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
      let caption = format!("{} · {} · {}", truncate(&prompt, 72), model, size);
      self.open_lightbox(tex, caption, file.clone());
    }

    // Caption row.
    ui.horizontal(|ui| {
      widgets::hud(ui, &format!("{} · {}", model, size), theme::FAINT);
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if widgets::ghost(ui, Icon::Folder, None, "Reveal in Explorer", false).clicked() {
          if let Some(path) = &file {
            let _ = open::that_detached(path.parent().unwrap_or(std::path::Path::new(".")));
          }
        }
        if widgets::ghost(ui, Icon::Plus, None, "Reuse prompt", false).clicked() {
          self.img_prompt = prompt.clone();
          ctx.request_repaint();
        }
      });
    });
  }
}

fn truncate(s: &str, n: usize) -> String {
  if s.chars().count() <= n {
    s.to_owned()
  } else {
    let t: String = s.chars().take(n).collect();
    format!("{t}…")
  }
}
