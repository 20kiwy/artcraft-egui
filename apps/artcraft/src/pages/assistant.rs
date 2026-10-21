//! Assistant: ZAI-backed chat sessions (create/switch/delete) with chat
//! bubbles and the send composer DOCKED to the bottom of the page so the
//! text area is always visible and reachable.

use egui::{vec2, Align, CornerRadius, FontId, Layout, Margin};

use crate::app::{ArtcraftApp, ChatLine};
use crate::icons::Icon;
use crate::theme;
use crate::widgets;

impl ArtcraftApp {
  pub(crate) fn show_assistant(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    // ---- docked composer -----------------------------------------------
    // Bottom panel first (reserves the bottom of the page — mirrors the
    // TypeScript app's chat input). Before the fix the composer was appended
    // after a full-height ScrollArea and ended up clipped off-screen.
    egui::Panel::bottom("asst-composer")
      .resizable(false)
      .frame(egui::Frame::NONE.outer_margin(Margin::same(8)))
      .show(ui, |ui| {
        widgets::prompt_box(ui, |ui| {
          let resp = egui::TextEdit::multiline(&mut self.asst.draft)
            .hint_text("Message ArtCraft Assistant…  (Enter to send, Shift+Enter for a new line)")
            .desired_rows(2)
            .desired_width(ui.available_width())
            .frame(egui::Frame::NONE)
            .font(FontId::new(14.5, theme::medium()))
            .show(ui);
          if self.focus_composer {
            self.focus_composer = false;
            resp.response.request_focus();
          }
          let enter = resp.response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !ui.input(|i| i.modifiers.shift);
          ui.add_space(4.0);
          ui.horizontal(|ui| {
            // Session controls sit with the composer, TS-chat style.
            session_controls(ui, self);
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
      });

    // ---- messages (fill everything above the composer) ------------------
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
  }
}

/// Session picker + new / refresh / delete, drawn once in the composer row.
fn session_controls(ui: &mut egui::Ui, app: &mut ArtcraftApp) {
  let active = app.asst.active.clone();
  let label = active
    .as_deref()
    .and_then(|id| app.asst.sessions.iter().find(|s| s.id == id))
    .map(|s| format!("{} · {} msg", s.title, s.message_count))
    .unwrap_or_else(|| "no session — created on first message".to_owned());
  egui::ComboBox::from_id_salt("asst-session")
    .width(220.0)
    .selected_text(&label)
    .show_ui(ui, |ui| {
      if app.asst.sessions.is_empty() {
        ui.add(egui::Label::new(egui::RichText::new("no sessions yet").color(theme::FAINT)));
      }
      for s in &app.asst.sessions {
        let id = s.id.clone();
        ui.selectable_value(&mut app.asst.active, Some(id.clone()), format!("{} · {} msg", s.title, s.message_count));
      }
    });
  let before = active.clone();
  // Selectable_value only changes the id — detect a switch and load it.
  if app.asst.active != before {
    if let Some(id) = app.asst.active.clone() {
      app.switch_session(id);
    }
  }

  if widgets::ghost(ui, Icon::Plus, None, "New chat", false).clicked() {
    app.new_session();
  }
  if widgets::ghost(ui, Icon::Refresh, None, "Refresh sessions", false).clicked() {
    app.queue_sessions();
  }
  if widgets::ghost(ui, Icon::Trash, None, "Delete current chat", false).clicked() {
    app.delete_active_session();
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
      // Hug short texts; wrap long ones before they can run off the window.
      // (A horizontal parent reports the whole row as available width, so an
      // explicit bound is required either way.)
      let max_w = (ui.available_width() * 0.72).min(760.0).max(220.0);
      let galley = ui
        .painter()
        .layout_no_wrap(line.content.clone(), FontId::new(14.0, egui::FontFamily::Proportional), ink);
      let w = galley.size().x.min(max_w);
      ui.allocate_ui_with_layout(vec2(w, 0.0), Layout::top_down(Align::LEFT), |ui| {
        ui.label(egui::RichText::new(&line.content).size(14.0).color(ink));
      });
    });
}
