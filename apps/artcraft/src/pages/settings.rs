//! Settings: plugin proxy URL, connection test, ZAI config readout, session
//! management and data-folder access.

use egui::{CornerRadius, FontId, Margin};

use crate::app::{ArtcraftApp, ConnState};
use crate::icons::Icon;
use crate::state;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("settings-scroll").show(ui, |ui| {
      ui.add_space(12.0);

      // --- Plugin proxy ---
      widgets::card(ui, |ui| {
        widgets::section_label(ui, "plugin proxy", theme::FAINT);
        ui.add_space(4.0);
        widgets::hud(ui, "hardcoded default — the address this app ships with", theme::FAINT);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
          let url_edit = egui::TextEdit::singleline(&mut self.draft_proxy_url)
            .hint_text(state::DEFAULT_PROXY_URL)
            .desired_width(ui.available_width() - 110.0)
            .font(FontId::new(13.5, theme::mono()));
          ui.add(url_edit);
          if widgets::primary(ui, Some(Icon::Plug), "Apply").clicked() {
            let url = self.draft_proxy_url.clone();
            self.apply_proxy_url(&url);
          }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
          let (text, color) = match &self.conn {
            ConnState::Checking => ("checking…".to_owned(), theme::DIM),
            ConnState::Online { mode } => (format!("online · {mode}"), if mode == "live" { theme::OK } else { theme::WARN }),
            ConnState::Offline { err } => (format!("offline — {err}"), theme::BAD),
          };
          widgets::kv(ui, "connection", &text, color);
        });
        ui.horizontal(|ui| {
          if widgets::ghost(ui, Icon::Refresh, Some("Test connection"), "GET /health", false).clicked() {
            self.conn = ConnState::Checking;
            self.queue_health();
            self.queue_config();
          }
          if widgets::ghost(ui, Icon::Plus, Some("Reset to default"), state::DEFAULT_PROXY_URL, false).clicked() {
            let d = state::DEFAULT_PROXY_URL.to_owned();
            self.apply_proxy_url(&d);
          }
        });
      });
      ui.add_space(8.0);

      // --- ZAI plugin readout ---
      widgets::card(ui, |ui| {
        widgets::section_label(ui, "zai plugin (read from /api/v1/config)", theme::FAINT);
        ui.add_space(6.0);
        match &self.cfg {
          Some(cfg) => {
            let backend = cfg.backend.clone().unwrap_or_default();
            widgets::kv(ui, "mode", &cfg.mode, if cfg.mode == "live" { theme::OK } else { theme::WARN });
            widgets::kv(ui, "zai base url", &cfg.zai_base_url, theme::MUTED);
            // Credentials live on the proxy side. A bridge backend means a
            // server holds the key — the desktop app intentionally has none.
            let (key_text, key_color) = if cfg.api_key_set {
              ("configured (on the proxy)", theme::OK)
            } else if backend.contains("bridge") {
              ("none — held server-side by the ZAI bridge", theme::OK)
            } else {
              ("missing — mock mode", theme::WARN)
            };
            widgets::kv(ui, "api key", key_text, key_color);
            if !backend.is_empty() {
              widgets::kv(ui, "backend", &backend, theme::MUTED);
            }
            widgets::kv(ui, "chat model", &cfg.chat_model, theme::MUTED);
            widgets::kv(ui, "image model", &cfg.image_model, theme::MUTED);
            ui.add_space(4.0);
            widgets::hud(ui, &format!("chat models: {}", cfg.chat_models.join(", ")), theme::FAINT);
            widgets::hud(ui, &format!("image models: {}", cfg.image_models.join(", ")), theme::FAINT);
          },
          None => {
            widgets::hud(ui, "no config — is the proxy running?", theme::FAINT);
          },
        }
      });
      ui.add_space(8.0);

      // --- Sessions ---
      widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
          widgets::section_label(ui, &format!("zai sessions ({})", self.asst.sessions.len()), theme::FAINT);
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::ghost(ui, Icon::Trash, Some("Clear all"), "Delete every session", false).clicked() {
              self.clear_sessions();
            }
            if widgets::ghost(ui, Icon::Refresh, None, "Refresh", false).clicked() {
              self.queue_sessions();
            }
          });
        });
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(180.0).id_salt("settings-sessions").show(ui, |ui| {
          if self.asst.sessions.is_empty() {
            widgets::hud(ui, "no sessions — they appear as you chat", theme::FAINT);
          }
          for s in &self.asst.sessions {
            egui::Frame::default()
              .fill(theme::SUNKEN)
              .corner_radius(CornerRadius::same(3))
              .inner_margin(Margin::symmetric(10, 6))
              .show(ui, |ui| {
                ui.horizontal(|ui| {
                  ui.label(egui::RichText::new(&s.title).size(13.0).color(theme::INK));
                  ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    widgets::hud(ui, &short_id(&s.id), theme::FAINT);
                    widgets::hud(ui, &format!("{} msg", s.message_count), theme::FAINT);
                  });
                });
              });
            ui.add_space(2.0);
          }
        });
      });
      ui.add_space(8.0);

      // --- Data ---
      widgets::card(ui, |ui| {
        widgets::section_label(ui, "data", theme::FAINT);
        ui.add_space(6.0);
        widgets::kv(ui, "library items", &self.library.len().to_string(), theme::MUTED);
        widgets::kv(ui, "folder", state::DATA_DIR, theme::MUTED);
        ui.horizontal(|ui| {
          if widgets::ghost(ui, Icon::Folder, Some("Open data folder"), "Reveal artcraft-data", false).clicked() {
            let _ = open::that_detached(state::data_dir());
          }
        });
      });
      ui.add_space(8.0);

      // --- About ---
      widgets::card(ui, |ui| {
        widgets::section_label(ui, "about", theme::FAINT);
        ui.add_space(6.0);
        widgets::kv(ui, "app", concat!("artcraft egui v", env!("CARGO_PKG_VERSION")), theme::MUTED);
        widgets::kv(ui, "theme", "artcraft-native · mit or apache-2.0", theme::MUTED);
        widgets::kv(ui, "fonts", "inter · archivo · geist mono (ofl)", theme::MUTED);
        widgets::kv(ui, "upstream", "github.com/storytold/artcraft", theme::MUTED);
        let _ = ctx;
      });

      ui.add_space(16.0);
    });
  }
}

fn short_id(id: &str) -> String {
  id.chars().take(8).collect()
}
