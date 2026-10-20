//! App-side state: settings persistence, the generated-images library and the
//! worker-thread event channel.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::proxy;

/// The hardcoded plugin connection detail: the egui app ships pointing at the
/// local `artcraft-proxy` (see `artcraft-proxy/src/config.rs` for the ZAI side).
pub const DEFAULT_PROXY_URL: &str = "http://127.0.0.1:8787";

/// Everything the app writes lands here, next to the executable's cwd.
pub const DATA_DIR: &str = "artcraft-data";

pub fn data_dir() -> PathBuf {
  PathBuf::from(DATA_DIR)
}

// ------------------------------------------------------------- settings ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
  pub proxy_url: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self { proxy_url: DEFAULT_PROXY_URL.to_owned() }
  }
}

impl Settings {
  pub fn load() -> Self {
    let path = data_dir().join("settings.json");
    std::fs::read_to_string(&path)
      .ok()
      .and_then(|s| serde_json::from_str(&s).ok())
      .unwrap_or_default()
  }

  pub fn save(&self) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("settings.json");
    if let Ok(json) = serde_json::to_string_pretty(self) {
      let _ = std::fs::write(path, json);
    }
  }
}

// -------------------------------------------------------------- library ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryEntry {
  pub id: String,
  pub prompt: String,
  pub model: String,
  pub size: String,
  /// File name inside `artcraft-data/generated/`.
  pub file: String,
  pub created_at: u64,
}

pub fn generated_dir() -> PathBuf {
  data_dir().join("generated")
}

pub fn load_library() -> Vec<LibraryEntry> {
  let path = data_dir().join("library.json");
  std::fs::read_to_string(&path)
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_default()
}

pub fn save_library(entries: &[LibraryEntry]) {
  let dir = data_dir();
  let _ = std::fs::create_dir_all(&dir);
  if let Ok(json) = serde_json::to_string_pretty(entries) {
    let _ = std::fs::write(dir.join("library.json"), json);
  }
}

pub fn entry_path(entry: &LibraryEntry) -> PathBuf {
  generated_dir().join(&entry.file)
}

// --------------------------------------------------------------- events ---

pub enum Event {
  Health(Result<proxy::Health, String>),
  Config(Result<proxy::ConfigInfo, String>),
  Sessions(Result<Vec<proxy::SessionInfo>, String>),
  SessionDetail(Result<proxy::SessionFull, String>),
  SessionCreated(Result<String, String>),
  Chat(Result<proxy::ChatReply, String>),
  Images(Result<Vec<proxy::ImageOut>, String>),
  Cleared(Result<usize, String>),
}

/// Runs `f` on a worker thread and delivers its [`Event`] to the UI loop.
pub fn spawn_event(tx: &Sender<Event>, f: impl FnOnce() -> Event + Send + 'static) {
  let tx = tx.clone();
  std::thread::spawn(move || {
    let _ = tx.send(f());
  });
}

/// Decodes a proxy `image_base64` payload into raw bytes.
pub fn decode_image_b64(b64: &str) -> anyhow::Result<Vec<u8>> {
  use base64::Engine;
  base64::engine::general_purpose::STANDARD
    .decode(b64.trim())
    .context("invalid base64 image payload")
}

pub fn now() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

/// Re-export so `app.rs` can hold the receiver without naming mpsc.
pub type EventRx = Receiver<Event>;
pub type EventTx = Sender<Event>;
