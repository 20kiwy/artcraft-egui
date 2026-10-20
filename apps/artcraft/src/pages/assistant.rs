//! Assistant: ZAI-backed chat sessions (create/switch/delete) with chat
//! bubbles and an Enter-to-send docked composer.

use egui::{Align, CornerRadius, FontId, Layout, Margin};

use crate::app::{ArtcraftApp, ChatLine};
use crate::icons::Icon;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_assistant(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    // ---- session controls ----
    ui.horizontal(|ui| {
      let active = self.asst.active.clone();
      let label = active
        .as_deref()
        .and_then(|id| self.asst.sessions.iter().find(|s| s.id == id))
        .map(|s| format!("{} · {} msg", s.title, s.message_count))
        .unwrap_or_else(|| "no session — one is created on first message".to_owned());
      egui::ComboBox::from_id_salt("asst-session")
        .width(300.0)
        .selected_text(&label)
        .show_ui(ui, |ui| {
          if self.asst.sessions.is_empty() {
            ui.add(egui::Label::new(egui::RichText::new("no sessions yet").color(theme::FAINT)));
          }
          for s in &self.asst.sessions {
            let id = s.id.clone();
            ui.selectable_value(&mut self.asst.active, Some(id.clone()), format!("{} · {} msg", s.title, s.message_count));
          }
        });
      let before = active.clone();
      // Selectable_value only changes the id — detect a switch and load it.
      if self.asst.active != before {
        if let Some(id) = self.asst.active.clone() {
          self.switch_session(id);
        }
      }

      if widgets::ghost(ui, Icon::Plus, None, "New chat", false).clicked() {
        self.new_session();
      }
      if widgets::ghost(ui, Icon::Refresh, None, "Refresh sessions", false).clicked() {
        self.queue_sessions();
      }
      if widgets::ghost(ui, Icon::Trash, None, "Delete current chat", false).clicked() {
        self.delete_active_session();
      }
    });

    // ---- messages ----
    egui::ScrollArea::vertical()
      .auto_shrink([false, false])
      .stick_to_bottom(true)
      .id_salt("asst-scroll")
      .show(ui, |ui| {
        ui.add_space(8.0);
        if self.asst.lines.is_empty() && !self.asst.busy {
          ui.add_space(36.0);
          ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Ask for ideas, prompts, critique — GLM answers here.").size(15.0).color(theme::MUTED));
            ui.add_space(6.0);
            let mode = match &self.conn {
              crate::app::ConnState::Online { mode } => mode.clone(),
              _ => "…".to_owned(),
            };
            widgets::hud(ui, &format!("glm via artcraft-proxy · mode: {mode}"), theme::FAINT);
          });
          ui.add_space(36.0);
        }
        for line in &self.asst.lines {
          bubble(ui, line);
        }
        if self.asst.busy {
          ui.horizontal(|ui| {
            ui.add_space(12.0);
            egui::Frame::default()
              .fill(theme::CONTROLS)
              .corner_radius(CornerRadius::same(8))
              .stroke(theme::hairline())
              .inner_margin(Margin::symmetric(12, 8))
              .show(ui, |ui| {
                ui.horizontal(|ui| {
                  ui.add(egui::Spinner::new().size(14.0));
                  ui.add_space(6.0);
                  widgets::hud(ui, "thinking…", theme::ACCENT_INK);
                });
              });
          });
        }
        ui.add_space(10.0);
      });

    // ---- composer ----
    ui.add_space(4.0);
    widgets::prompt_box(ui, |ui| {
      let resp = egui::TextEdit::multiline(&mut self.asst.draft)
        .hint_text("Message ArtCraft Assistant… (Enter to send, Shift+Enter for a new line)")
        .desired_rows(2)
        .desired_width(ui.available_width())
        .frame(egui::Frame::NONE)
        .font(FontId::new(14.5, theme::medium()))
        .show(ui);
      let enter = resp.response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !ui.input(|i| i.modifiers.shift);
      ui.add_space(4.0);
      ui.horizontal(|ui| {
        widgets::hud(ui, "sessions stored by the proxy · history sent every turn", theme::FAINT);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
          if self.asst.busy {
            ui.add(egui::Spinner::new().size(16.0));
            ui.add_space(6.0);
            widgets::hud(ui, "sending", theme::MUTED);
          } else if widgets::primary(ui, Some(Icon::Send), "Send").clicked() || enter {
            self.send_chat(ctx);
          }
        });
      });
    });
  }
}

fn bubble(ui: &mut egui::Ui, line: &ChatLine) {
  let is_user = line.role == "user";
  let (fill, ink) = if is_user {
    (theme::fade(theme::ACCENT, 0.18), theme::INK_STRONG)
  } else {
    (theme::CONTROLS, theme::INK)
  };
  ui.horizontal(|ui| {
    if is_user {
      ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        bubble_frame(ui, fill, ink, line);
      });
    } else {
      bubble_frame(ui, fill, ink, line);
    }
  });
  ui.add_space(2.0);
  if let Some(meta) = &line.meta {
    ui.horizontal(|ui| {
      ui.add_space(12.0);
      widgets::hud(ui, meta, theme::FAINT);
    });
  }
  ui.add_space(2.0);
}

fn bubble_frame(ui: &mut egui::Ui, fill: egui::Color32, ink: egui::Color32, line: &ChatLine) {
  egui::Frame::default()
    .fill(fill)
    .corner_radius(CornerRadius::same(8))
    .stroke(theme::hairline())
    .inner_margin(Margin::same(10))
    .show(ui, |ui| {
      ui.set_max_width(ui.available_width() * 0.72);
      ui.label(egui::RichText::new(&line.content).size(14.0).color(ink));
    });
}
