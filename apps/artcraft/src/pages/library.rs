//! Library: every generated image saved on disk, with lazy texture loading.

use egui::{Color32, CornerRadius, Sense, Stroke, StrokeKind, TextureOptions, vec2};

use crate::app::ArtcraftApp;
use crate::icons::{self, Icon};
use crate::state;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
      widgets::hud(ui, &format!("{} item(s) · artcraft-data/generated", self.library.len()), theme::FAINT);
      ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        if widgets::ghost(ui, Icon::Refresh, None, "Reload from disk", false).clicked() {
          self.library = state::load_library();
          self.tex_cache.clear();
        }
        if widgets::ghost(ui, Icon::Folder, Some("Open folder"), "Reveal artcraft-data", false).clicked() {
          let _ = open::that_detached(state::generated_dir());
        }
      });
    });
    widgets::hairline(ui);
    ui.add_space(8.0);

    if self.library.is_empty() {
      ui.add_space(48.0);
      ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Nothing here yet — craft something in the Image studio.").size(15.0).color(theme::MUTED));
      });
      return;
    }

    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("library-scroll").show(ui, |ui| {
      let cols = ((ui.available_width() / 300.0).floor() as usize).clamp(1, 4);
      egui::Grid::new("library-grid").spacing(vec2(10.0, 14.0)).show(ui, |ui| {
        for i in 0..self.library.len() {
          self.library_tile(ui, i);
          if (i + 1) % cols == 0 || i + 1 == self.library.len() {
            ui.end_row();
          }
        }
      });
      ui.add_space(12.0);
    });
  }

  fn library_tile(&mut self, ui: &mut egui::Ui, idx: usize) {
    let Some(entry) = self.library.get(idx).cloned() else { return };

    // Lazy texture load.
    let tex = match self.tex_cache.get(&entry.id) {
      Some(t) => t.clone(),
      None => match load_texture(ui, &entry) {
        Some(t) => {
          self.tex_cache.insert(entry.id.clone(), t.clone());
          t
        },
        None => return,
      },
    };

    let tile_w = ((ui.available_width() / 4.0) - 20.0).clamp(180.0, 280.0);
    let fit = icons::fit(tex.size_vec2(), vec2(tile_w, tile_w * 0.85));
    let (rect, resp) = ui.allocate_exact_size(fit, Sense::click());
    ui.painter().rect(rect, CornerRadius::same(8), Color32::TRANSPARENT, Stroke::new(1.0, if resp.hovered() { theme::ACCENT_400 } else { theme::LINE }), StrokeKind::Inside);
    egui::Image::new(&tex).corner_radius(theme::RADIUS).paint_at(ui, rect.shrink(1.0));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, &entry.prompt));
    if resp.hovered() {
      ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
      let caption = format!("{} · {} · {}", entry.model, entry.size, entry.file);
      self.open_lightbox(tex, caption, Some(state::entry_path(&entry)));
    }
    ui.horizontal(|ui| {
      widgets::hud(ui, &entry.model, theme::FAINT);
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if widgets::ghost(ui, Icon::Folder, None, "Reveal in Explorer", false).clicked() {
          let _ = open::that_detached(state::generated_dir());
        }
      });
    });
  }
}

fn load_texture(ui: &egui::Ui, entry: &state::LibraryEntry) -> Option<egui::TextureHandle> {
  let bytes = std::fs::read(state::entry_path(entry)).ok()?;
  let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
  let size = [img.width() as usize, img.height() as usize];
  let color = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
  Some(ui.ctx().load_texture(format!("lib-{}", entry.id), color, TextureOptions::LINEAR))
}
