//! Home: hero, plugin status cards and quickstart.

use egui::{Color32, CornerRadius, FontId, Layout, Margin};

use crate::app::ArtcraftApp;
use crate::shell::Page;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_home(&mut self, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
      ui.add_space(28.0);

      // Hero.
      ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("ArtCraft").font(FontId::new(44.0, theme::display())).color(theme::INK_STRONG));
        ui.add_space(6.0);
        ui.label(
          egui::RichText::new("The IDE for artists — rebuilt in pure Rust with egui. No TypeScript. No webview.")
            .size(15.0)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
          ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
            widgets::badge(ui, "egui 0.36", theme::BADGE_BETA);
            widgets::badge(ui, "artcraft-proxy · zai", theme::BADGE_BETA);
            widgets::badge(ui, "mit / apache-2.0", theme::BADGE_BETA);
          });
        });
      });

      ui.add_space(24.0);

      // Actions.
      ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
          let gen = widgets::primary(ui, Some(crate::icons::Icon::Wand), "Open Image Studio");
          if gen.clicked() {
            self.page = Page::CreateImage;
          }
          ui.add_space(8.0);
          let chat = widgets::primary(ui, Some(crate::icons::Icon::Chat), "Open Assistant");
          if chat.clicked() {
            self.page = Page::Assistant;
          }
        });
      });

      ui.add_space(28.0);

      // Status cards.
      let conn = self.conn.clone();
      let cfg = self.cfg.clone();
      let cards: [(&str, Box<dyn FnOnce(&mut egui::Ui) + '_>); 3] = [
        (
          "AI PLUGIN",
          Box::new(move |ui: &mut egui::Ui| {
            let (text, color) = match &conn {
              crate::app::ConnState::Checking => ("checking…".to_owned(), theme::DIM),
              crate::app::ConnState::Online { mode } => (format!("online · {mode}"), if mode == "live" { theme::OK } else { theme::WARN }),
              crate::app::ConnState::Offline { err } => (format!("offline — {err}"), theme::BAD),
            };
            widgets::kv(ui, "status", &text, color);
            widgets::kv(ui, "proxy", "http://127.0.0.1:8787", theme::MUTED);
            if let Some(cfg) = &cfg {
              widgets::kv(ui, "chat model", &cfg.chat_model, theme::MUTED);
              widgets::kv(ui, "image model", &cfg.image_model, theme::MUTED);
            }
            ui.add_space(6.0);
            widgets::hud(ui, "glm-4.6 answers · cogview-4 paints · sessions persist", theme::FAINT);
          }),
        ),
        (
          "QUICKSTART",
          Box::new(|ui: &mut egui::Ui| {
            mono_line(ui, "$ cargo run -p artcraft-proxy    # start the ZAI plugin bridge");
            mono_line(ui, "$ cargo run -p artcraft            # this app");
            ui.add_space(8.0);
            widgets::hud(ui, "set ZAI_API_KEY on the proxy for live mode;", theme::FAINT);
            widgets::hud(ui, "without a key it runs in mock mode (still fully usable)", theme::FAINT);
          }),
        ),
        (
          "UPSTREAM",
          Box::new(|ui: &mut egui::Ui| {
            widgets::kv(ui, "clone/artcraft", "v0.41.x · mit or apache-2.0", theme::MUTED);
            widgets::kv(ui, "clone/artcraftx", "companion client · unlicensed", theme::MUTED);
            ui.add_space(6.0);
            widgets::hud(ui, "theme + fonts mirrored from artcraft-native", theme::FAINT);
          }),
        ),
      ];

      let widths = ui.available_width();
      let gap = 8.0;
      let card_w = ((widths - 2.0 * gap) / 3.0).min(380.0);
      ui.horizontal(|ui| {
        for (title, body) in cards {
          egui::Frame::default()
            .fill(theme::PANEL)
            .corner_radius(CornerRadius::same(8))
            .stroke(theme::hairline())
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
              ui.set_width(card_w);
              widgets::section_label(ui, title, theme::FAINT);
              ui.add_space(4.0);
              body(ui);
            });
        }
      });

      ui.add_space(16.0);
      ui.vertical_centered(|ui| {
        widgets::hud(ui, "artcraft egui rebuild — sessions, chat + image studio, zero typescript", theme::FAINT);
      });
      let _ = Color32::TRANSPARENT;
    });
  }
}

fn mono_line(ui: &mut egui::Ui, line: &str) {
  egui::Frame::default()
    .fill(theme::SUNKEN)
    .corner_radius(theme::RADIUS)
    .inner_margin(Margin::symmetric(10, 6))
    .show(ui, |ui| {
      ui.label(egui::RichText::new(line).font(FontId::new(12.0, theme::mono())).color(theme::ACCENT_INK));
    });
}
