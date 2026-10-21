//! Home: the TypeScript app's landing — a big centered "what will you
//! create?" composer with Image/Chat mode chips that route into the studio —
//! plus plugin status and quickstart cards.

use egui::{Align, Color32, CornerRadius, FontId, Layout, Margin, Rect, Sense, Stroke, StrokeKind, vec2, pos2};

use crate::app::ArtcraftApp;
use crate::icons::{self, Icon};
use crate::shell::Page;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_home(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
      ui.add_space(36.0);

      // Hero.
      ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("ArtCraft").font(FontId::new(44.0, theme::display())).color(theme::INK_STRONG));
        ui.add_space(6.0);
        ui.label(
          egui::RichText::new("The IDE for artists — rebuilt in pure Rust with egui. No TypeScript. No webview.")
            .size(15.0)
            .color(theme::MUTED),
        );
      });

      ui.add_space(22.0);

      // ---- hero composer (TypeScript landing style) ----
      let width = (ui.available_width() - 32.0).min(660.0);
      ui.vertical_centered(|ui| {
        ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Center), |ui| {
          // Mode chips above the box, like the web app's tool tabs.
          ui.horizontal(|ui| {
            ui.with_layout(Layout::left_to_right(Align::Center).with_main_align(Align::Center), |ui| {
              if mode_chip(ui, "Image", Icon::Wand, self.home_image_mode).clicked() {
                self.home_image_mode = true;
              }
              ui.add_space(6.0);
              if mode_chip(ui, "Chat", Icon::Chat, !self.home_image_mode).clicked() {
                self.home_image_mode = false;
              }
            });
          });
          ui.add_space(8.0);

          widgets::prompt_box(ui, |ui| {
            let (hint, font) = if self.home_image_mode {
              ("Describe what you want to create…", 15.5)
            } else {
              ("Ask the ArtCraft Assistant…", 15.5)
            };
            let resp = egui::TextEdit::multiline(&mut self.home_prompt)
              .hint_text(hint)
              .desired_rows(3)
              .desired_width(ui.available_width())
              .frame(egui::Frame::NONE)
              .font(FontId::new(font, theme::medium()))
              .show(ui);
            let enter = resp.response.lost_focus()
              && ui.input(|i| i.key_pressed(egui::Key::Enter))
              && !ui.input(|i| i.modifiers.shift);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
              widgets::hud(ui, "Enter to launch · options live on the studio pages", theme::FAINT);
              ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let launch = if self.home_image_mode {
                  widgets::primary(ui, Some(Icon::Wand), "Craft Image")
                } else {
                  widgets::primary(ui, Some(Icon::Send), "Send to Assistant")
                };
                if launch.clicked() || enter {
                  self.launch_home(ctx);
                }
              });
            });
          });
        });
      });

      ui.add_space(30.0);

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
              if let Some(backend) = &cfg.backend {
                widgets::kv(ui, "backend", backend, theme::MUTED);
              }
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
            widgets::hud(ui, "live with ZAI_API_KEY — or with --bridge-url so a", theme::FAINT);
            widgets::hud(ui, "server holds the key; otherwise mock mode (still usable)", theme::FAINT);
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
      // Fixed-width child UIs: kv rows inside must measure against the card,
      // not the whole content row (otherwise values overlap the card edge).
      ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
        for (title, body) in cards {
          ui.allocate_ui_with_layout(vec2(card_w, 100.0), Layout::top_down(Align::LEFT), |ui| {
            egui::Frame::default()
              .fill(theme::PANEL)
              .corner_radius(CornerRadius::same(8))
              .stroke(theme::hairline())
              .inner_margin(Margin::same(14))
              .show(ui, |ui| {
                widgets::section_label(ui, title, theme::FAINT);
                ui.add_space(4.0);
                body(ui);
              });
          });
          ui.add_space(gap);
        }
      });

      ui.add_space(16.0);
      ui.vertical_centered(|ui| {
        widgets::hud(ui, "artcraft egui rebuild — sessions, chat + image studio, zero typescript", theme::FAINT);
      });
      let _ = Color32::TRANSPARENT;
    });
  }

  /// Hero composer → route into the chosen studio and fire the request.
  fn launch_home(&mut self, ctx: &egui::Context) {
    let prompt = self.home_prompt.trim().to_owned();
    if prompt.is_empty() {
      self.toast("describe what you want to create first", true);
      return;
    }
    self.home_prompt.clear();
    if self.home_image_mode {
      self.img_prompt = prompt;
      self.page = Page::CreateImage;
      self.generate_images(ctx);
    } else {
      self.asst.draft = prompt;
      self.page = Page::Assistant;
      self.send_chat(ctx);
    }
  }
}

/// A pill-style tool tab (`Image` / `Chat`), TS landing style.
fn mode_chip(ui: &mut egui::Ui, label: &str, icon: Icon, selected: bool) -> egui::Response {
  let galley = ui.painter().layout_no_wrap(label.to_owned(), FontId::new(13.0, theme::semibold()), Color32::WHITE);
  let size = vec2(14.0 + 15.0 + 6.0 + galley.size().x + 14.0, 30.0);
  let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
  let p = ui.painter();
  let stroke = if selected {
    Stroke::new(1.0, theme::ACCENT_400)
  } else {
    Stroke::new(1.0, theme::LINE)
  };
  p.rect(rect, theme::RADIUS, if selected { theme::fade(theme::ACCENT, 0.16) } else { theme::WASH }, stroke, StrokeKind::Inside);
  let ink = if selected { theme::ACCENT_INK } else { theme::MUTED };
  icons::paint(p, Rect::from_center_size(pos2(rect.left() + 14.0 + 7.5, rect.center().y), vec2(15.0, 15.0)), icon, ink);
  p.galley_with_override_text_color(pos2(rect.left() + 14.0 + 15.0 + 6.0, rect.center().y - galley.size().y / 2.0), galley, ink);
  resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
  resp.on_hover_cursor(egui::CursorIcon::PointingHand)
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
