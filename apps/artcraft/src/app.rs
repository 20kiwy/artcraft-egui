//! The application: panels (title bar, sidebar, header, status bar), page
//! dispatch, worker-event polling and all user actions.
//!
//! Uses the eframe 0.36 split: [`eframe::App::logic`] for background/event
//! work and [`eframe::App::ui`] for rendering into the root `Ui` (panels now
//! nest inside a `Ui`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use egui::{Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Rect, Sense, Stroke, TextureHandle, Ui, vec2};

use crate::proxy::ProxyClient;
use crate::shell::{self, Page};
use crate::state::{self, Event, LibraryEntry, Settings};
use crate::theme;
use crate::titlebar;
use crate::widgets;

pub struct ArtcraftApp {
  pub(crate) tx: state::EventTx,
  pub(crate) rx: state::EventRx,

  pub(crate) page: Page,

  // Connection
  pub(crate) settings: Settings,
  pub(crate) client: std::sync::Arc<ProxyClient>,
  pub(crate) conn: ConnState,
  pub(crate) next_health: Instant,
  pub(crate) cfg: Option<crate::proxy::ConfigInfo>,
  pub(crate) draft_proxy_url: String,

  // Image studio
  pub(crate) img_prompt: String,
  pub(crate) img_model: Option<String>,
  pub(crate) img_size: String,
  pub(crate) img_count: usize,
  pub(crate) img_busy: bool,
  pub(crate) feed: Vec<GenItem>,

  // Assistant
  pub(crate) asst: AssistantState,

  // Library
  pub(crate) library: Vec<LibraryEntry>,
  pub(crate) tex_cache: HashMap<String, TextureHandle>,

  // Overlays
  pub(crate) lightbox: Option<Lightbox>,
  pub(crate) toast: Option<Toast>,
}

#[derive(Clone, Debug)]
pub enum ConnState {
  Checking,
  Online { mode: String },
  Offline { err: String },
}

pub struct GenItem {
  pub tex: TextureHandle,
  pub prompt: String,
  pub model: String,
  pub size: String,
  #[allow(dead_code)]
  pub created_at: u64,
  pub file: Option<PathBuf>,
}

pub struct ChatLine {
  pub role: String,
  pub content: String,
  pub meta: Option<String>,
}

pub struct AssistantState {
  pub sessions: Vec<crate::proxy::SessionInfo>,
  pub active: Option<String>,
  pub lines: Vec<ChatLine>,
  pub draft: String,
  pub busy: bool,
}

#[derive(Clone)]
pub struct Lightbox {
  pub tex: TextureHandle,
  pub caption: String,
  pub file: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Toast {
  pub text: String,
  pub err: bool,
  pub born: Instant,
}

impl ArtcraftApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    theme::apply(&cc.egui_ctx);
    let (tx, rx) = channel();
    let settings = Settings::load();
    let client = std::sync::Arc::new(ProxyClient::new(&settings.proxy_url));
    let library = state::load_library();
    let mut app = Self {
      tx,
      rx,
      page: Page::Home,
      settings,
      client,
      conn: ConnState::Checking,
      next_health: Instant::now(),
      cfg: None,
      draft_proxy_url: String::new(),
      img_prompt: String::new(),
      img_model: None,
      img_size: "1024x1024".to_owned(),
      img_count: 1,
      img_busy: false,
      feed: Vec::new(),
      asst: AssistantState { sessions: Vec::new(), active: None, lines: Vec::new(), draft: String::new(), busy: false },
      library,
      tex_cache: HashMap::new(),
      lightbox: None,
      toast: None,
    };
    app.draft_proxy_url = app.settings.proxy_url.clone();
    app.queue_health();
    app.queue_config();
    app.queue_sessions();
    app
  }

  // ------------------------------------------------------------- events ---

  pub(crate) fn toast(&mut self, text: impl Into<String>, err: bool) {
    self.toast = Some(Toast { text: text.into(), err, born: Instant::now() });
  }

  pub(crate) fn queue_health(&self) {
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::Health(cl.health().map_err(|e| e.to_string())));
  }

  pub(crate) fn queue_config(&self) {
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::Config(cl.config().map_err(|e| e.to_string())));
  }

  pub(crate) fn queue_sessions(&self) {
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::Sessions(cl.sessions().map_err(|e| e.to_string())));
  }

  fn poll_events(&mut self, ctx: &egui::Context) {
    while let Ok(ev) = self.rx.try_recv() {
      match ev {
        Event::Health(r) => match r {
          Ok(h) if h.ok => self.conn = ConnState::Online { mode: h.mode },
          Ok(_) => self.conn = ConnState::Offline { err: "proxy reported ok=false".into() },
          Err(e) => self.conn = ConnState::Offline { err: e },
        },
        Event::Config(r) => self.cfg = r.ok(),
        Event::Sessions(r) => {
          if let Ok(list) = r {
            self.asst.sessions = list;
          }
        },
        Event::SessionDetail(r) => {
          if let Ok(s) = r {
            if self.asst.active.as_deref() == Some(s.id.as_str()) {
              self.asst.lines = s.messages.into_iter().map(|m| ChatLine { role: m.role, content: m.content, meta: None }).collect();
            }
          }
        },
        Event::SessionCreated(r) => match r {
          Ok(id) => {
            self.asst.active = Some(id);
            self.asst.lines.clear();
            self.queue_sessions();
          },
          Err(e) => self.toast(format!("could not create session: {e}"), true),
        },
        Event::Chat(r) => {
          self.asst.busy = false;
          match r {
            Ok(rep) => {
              if self.asst.active.is_none() {
                self.asst.active = Some(rep.session_id.clone());
              }
              self.asst.lines.push(ChatLine {
                role: "assistant".into(),
                content: rep.reply,
                meta: Some(format!("{} · {}", rep.model, rep.mode)),
              });
              self.queue_sessions();
            },
            Err(e) => self.toast(format!("chat failed: {e}"), true),
          }
        },
        Event::Images(r) => {
          self.img_busy = false;
          match r {
            Ok(outs) => {
              let n = outs.len();
              for out in outs {
                self.accept_image(ctx, out);
              }
              if n > 0 {
                self.toast(format!("{n} image(s) crafted → Library"), false);
              }
            },
            Err(e) => self.toast(format!("image generation failed: {e}"), true),
          }
        },
        Event::Cleared(r) => match r {
          Ok(n) => {
            self.asst.active = None;
            self.asst.lines.clear();
            self.toast(format!("cleared {n} session(s)"), false);
            self.queue_sessions();
          },
          Err(e) => self.toast(format!("clear failed: {e}"), true),
        },
      }
    }
  }

  fn accept_image(&mut self, ctx: &egui::Context, out: crate::proxy::ImageOut) {
    let bytes = match state::decode_image_b64(&out.image_base64) {
      Ok(b) => b,
      Err(e) => {
        self.toast(format!("bad image payload: {e}"), true);
        return;
      },
    };
    let img = match image::load_from_memory(&bytes) {
      Ok(i) => i.to_rgba8(),
      Err(e) => {
        self.toast(format!("cannot decode image: {e}"), true);
        return;
      },
    };
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    let tex = ctx.load_texture(format!("gen-{}-{}", state::now(), self.feed.len()), color, egui::TextureOptions::LINEAR);

    // Persist into the local library.
    let id = uuid::Uuid::new_v4().to_string();
    let file = format!("{id}.png");
    let path = state::generated_dir().join(&file);
    let saved = std::fs::create_dir_all(state::generated_dir()).is_ok() && std::fs::write(&path, &bytes).is_ok();
    if saved {
      self.library.insert(0, LibraryEntry {
        id,
        prompt: out.prompt.clone(),
        model: out.model.clone(),
        size: out.size.clone(),
        file,
        created_at: state::now(),
      });
      if self.library.len() > 200 {
        self.library.truncate(200);
      }
      state::save_library(&self.library);
    }

    self.feed.insert(0, GenItem { tex, prompt: out.prompt, model: out.model, size: out.size, created_at: state::now(), file: saved.then_some(path) });
    if self.feed.len() > 24 {
      self.feed.pop();
    }
  }

  fn maybe_health(&mut self, ctx: &egui::Context) {
    if Instant::now() >= self.next_health {
      self.queue_health();
      self.next_health = Instant::now() + Duration::from_secs(15);
      ctx.request_repaint_after(Duration::from_secs(15));
    }
  }

  // ------------------------------------------------------------ actions ---

  pub(crate) fn apply_proxy_url(&mut self, url: &str) {
    let url = url.trim().trim_end_matches('/').to_owned();
    if url.is_empty() {
      return;
    }
    self.settings = Settings { proxy_url: url };
    self.settings.save();
    self.client = std::sync::Arc::new(ProxyClient::new(&self.settings.proxy_url));
    self.draft_proxy_url = self.settings.proxy_url.clone();
    self.conn = ConnState::Checking;
    self.cfg = None;
    self.queue_health();
    self.queue_config();
    self.queue_sessions();
    self.toast("plugin proxy updated", false);
  }

  pub(crate) fn generate_images(&mut self, ctx: &egui::Context) {
    if self.img_busy {
      return;
    }
    let prompt = self.img_prompt.trim().to_owned();
    if prompt.is_empty() {
      self.toast("describe an image first", true);
      return;
    }
    self.img_busy = true;
    let cl = self.client.clone();
    let (count, size, model) = (self.img_count, self.img_size.clone(), self.img_model.clone());
    state::spawn_event(&self.tx, move || {
      let mut outs = Vec::new();
      for _ in 0..count {
        match cl.image(&prompt, &size, model.as_deref()) {
          Ok(o) => outs.push(o),
          Err(e) => return Event::Images(Err(e.to_string())),
        }
      }
      Event::Images(Ok(outs))
    });
    ctx.request_repaint_after(Duration::from_millis(120));
  }

  pub(crate) fn send_chat(&mut self, ctx: &egui::Context) {
    if self.asst.busy {
      return;
    }
    let msg = self.asst.draft.trim().to_owned();
    if msg.is_empty() {
      return;
    }
    self.asst.draft.clear();
    self.asst.lines.push(ChatLine { role: "user".into(), content: msg.clone(), meta: None });
    self.asst.busy = true;
    let cl = self.client.clone();
    let session = self.asst.active.clone();
    state::spawn_event(&self.tx, move || {
      // No session yet → the worker creates one, then chats on it.
      let session = match session {
        Some(s) => s,
        None => match cl.create_session() {
          Ok(s) => s,
          Err(e) => return Event::Chat(Err(e.to_string())),
        },
      };
      Event::Chat(cl.chat(&session, &msg, None).map_err(|e| e.to_string()))
    });
    ctx.request_repaint_after(Duration::from_millis(120));
  }

  pub(crate) fn new_session(&mut self) {
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::SessionCreated(cl.create_session().map_err(|e| e.to_string())));
  }

  pub(crate) fn switch_session(&mut self, id: String) {
    self.asst.active = Some(id.clone());
    self.asst.lines.clear();
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::SessionDetail(cl.session(&id).map_err(|e| e.to_string())));
  }

  pub(crate) fn delete_active_session(&mut self) {
    let Some(id) = self.asst.active.take() else { return };
    self.asst.lines.clear();
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || {
      if let Err(e) = cl.delete_session(&id) {
        eprintln!("[artcraft] delete session failed: {e}");
      }
      Event::Sessions(cl.sessions().map_err(|e| e.to_string()))
    });
  }

  pub(crate) fn clear_sessions(&mut self) {
    let cl = self.client.clone();
    state::spawn_event(&self.tx, move || Event::Cleared(cl.clear_sessions().map_err(|e| e.to_string())));
  }

  pub(crate) fn open_lightbox(&mut self, tex: TextureHandle, caption: String, file: Option<PathBuf>) {
    self.lightbox = Some(Lightbox { tex, caption, file });
  }

  // ------------------------------------------------------------- panels ---

  fn sidebar(&mut self, ui: &mut Ui) {
    egui::Panel::left("sidebar")
      .exact_size(shell::SIDEBAR_WIDTH)
      .resizable(false)
      .frame(egui::Frame::NONE.fill(theme::CHROME))
      .show(ui, |ui| {
        ui.add_space(14.0);
        self.nav_row(ui, Page::Home);
        ui.add_space(8.0);
        for (label, items) in shell::SECTIONS {
          widgets::section_label(ui, label, theme::FAINT);
          for p in items {
            self.nav_row(ui, *p);
          }
          ui.add_space(6.0);
        }
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
          ui.add_space(12.0);
          widgets::hud(ui, concat!("egui rebuild · v", env!("CARGO_PKG_VERSION")), theme::FAINT);
          widgets::hud(ui, "upstream: storytold/artcraft", theme::FAINT);
        });
      });
  }

  fn nav_row(&mut self, ui: &mut Ui, page: Page) {
    let selected = self.page == page;
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    let fill = if selected {
      theme::WASH_SELECTED
    } else if resp.hovered() {
      theme::WASH_HOVER
    } else {
      Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, theme::RADIUS, fill);
    let ink = if selected { theme::INK_STRONG } else { theme::SIDEBAR_INK };
    crate::icons::paint(ui.painter(), Rect::from_center_size(egui::pos2(rect.left() + 24.0, rect.center().y), vec2(16.0, 16.0)), page.icon(), ink);
    ui.painter().text(egui::pos2(rect.left() + 44.0, rect.center().y), Align2::LEFT_CENTER, page.label(), FontId::new(14.0, theme::medium()), ink);
    if selected {
      ui.painter().rect_filled(Rect::from_min_size(egui::pos2(rect.left() + 6.0, rect.center().y - 7.0), vec2(3.0, 14.0)), CornerRadius::same(2), theme::ACCENT);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, page.label()));
    if resp.clicked() {
      self.page = page;
    }
    if resp.hovered() {
      ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
  }

  fn statusbar(&mut self, ui: &mut Ui) {
    egui::Panel::bottom("statusbar")
      .exact_size(26.0)
      .frame(egui::Frame::NONE.fill(theme::CHROME).inner_margin(Margin::symmetric(12, 4)))
      .show(ui, |ui| {
        ui.horizontal_centered(|ui| {
          widgets::hud(ui, &format!("plugin: {}", self.settings.proxy_url), theme::FAINT);
          ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            widgets::hud(ui, "zai sessions · glm + cogview", theme::FAINT);
            widgets::hud(ui, "·", theme::FAINT);
            widgets::hud(ui, "artcraft-proxy", theme::FAINT);
          });
        });
      });
  }

  fn page_body(&mut self, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    egui::CentralPanel::default()
      .frame(egui::Frame::NONE.fill(theme::CHROME).inner_margin(Margin::same(shell::PANEL_GAP)))
      .show(ui, |ui| {
        // The rounded content card: header + page.
        egui::Frame::new()
          .fill(theme::BG)
          .stroke(theme::hairline())
          .corner_radius(CornerRadius::same(shell::PANEL_RADIUS))
          .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            egui::Panel::top("page-header")
              .exact_size(shell::HEADER_HEIGHT)
              .resizable(false)
              .frame(egui::Frame::NONE)
              .show(ui, |ui| self.header_inner(ui));
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
              match self.page {
                Page::Home => self.show_home(ui),
                Page::CreateImage => self.show_image(ui, &ctx),
                Page::Assistant => self.show_assistant(ui, &ctx),
                Page::Library => self.show_library(ui),
                Page::Settings => self.show_settings(ui, &ctx),
              }
            });
          });
      });
  }

  fn header_inner(&mut self, ui: &mut Ui) {
    let (section, label) = self.page.crumbs();
    let conn = self.conn.clone();
    ui.allocate_ui_with_layout(vec2(ui.available_width(), shell::HEADER_HEIGHT), Layout::left_to_right(Align::Center), |ui| {
      widgets::hud(ui, section, theme::FAINT);
      widgets::hud(ui, "›", theme::FAINT);
      ui.label(egui::RichText::new(label).font(FontId::new(14.0, theme::semibold())).color(theme::INK));
      ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let (text, dot_color) = match &conn {
          ConnState::Checking => ("checking", theme::DIM),
          ConnState::Online { mode } => (if mode == "live" { "live" } else { "mock" }, if mode == "live" { theme::OK } else { theme::WARN }),
          ConnState::Offline { .. } => ("offline", theme::BAD),
        };
        widgets::hud(ui, text, theme::MUTED);
        let (d, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        widgets::dot(ui.painter(), d.center(), dot_color);
      });
    });
  }

  fn toast_ui(&mut self, ctx: &egui::Context) {
    let Some(t) = self.toast.clone() else { return };
    egui::Area::new(Id::new("toast-area"))
      .anchor(Align2::RIGHT_TOP, [-16.0, 56.0])
      .order(egui::Order::Foreground)
      .show(ctx, |ui| {
        egui::Frame::new()
          .fill(theme::PANEL)
          .corner_radius(theme::RADIUS)
          .stroke(Stroke::new(1.0, if t.err { theme::BAD } else { theme::LINE }))
          .inner_margin(Margin::symmetric(14, 10))
          .shadow(theme::shadow())
          .show(ui, |ui| {
            ui.label(egui::RichText::new(&t.text).size(13.0).color(if t.err { theme::BAD } else { theme::INK }));
          });
      });
  }
}

impl eframe::App for ArtcraftApp {
  fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    self.poll_events(ctx);
    self.maybe_health(ctx);
    if let Some(t) = &self.toast {
      if t.born.elapsed() >= Duration::from_secs(4) {
        self.toast = None;
      } else {
        ctx.request_repaint_after(Duration::from_millis(250));
      }
    }
  }

  fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
    let ctx = ui.ctx().clone();
    if titlebar::render(ui, &self.conn, &self.settings.proxy_url) {
      self.page = Page::Settings;
    }
    self.sidebar(ui);
    self.page_body(ui);
    self.statusbar(ui);
    crate::lightbox::show(self, &ctx);
    self.toast_ui(&ctx);

    if self.img_busy || self.asst.busy {
      ctx.request_repaint_after(Duration::from_millis(120));
    }
  }
}
