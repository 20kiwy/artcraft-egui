//! In-memory chat-session store with JSON persistence.
//!
//! Sessions are the "connected sessions" the ArtCraft egui app talks to:
//! each holds an ordered message history that the proxy forwards to ZAI
//! (plus a system prompt) for every turn.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

pub fn now() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMsg {
  pub role: String,
  pub content: String,
  pub ts: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
  pub id: String,
  pub title: String,
  pub created_at: u64,
  pub updated_at: u64,
  pub messages: Vec<ChatMsg>,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
  pub id: String,
  pub title: String,
  pub created_at: u64,
  pub updated_at: u64,
  pub message_count: usize,
}

impl Session {
  fn new() -> Self {
    let ts = now();
    Self { id: Uuid::new_v4().to_string(), title: "New chat".into(), created_at: ts, updated_at: ts, messages: Vec::new() }
  }

  fn info(&self) -> SessionInfo {
    SessionInfo {
      id: self.id.clone(),
      title: self.title.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
      message_count: self.messages.len(),
    }
  }
}

pub struct SessionStore {
  inner: Mutex<BTreeMap<String, Session>>,
  path: PathBuf,
}

impl SessionStore {
  /// Loads `dir/sessions.json` (best effort — a missing or corrupt file just
  /// starts a fresh store).
  pub fn load(dir: &Path) -> Self {
    let path = dir.join("sessions.json");
    let mut map = BTreeMap::new();
    match std::fs::read_to_string(&path).ok().map(|s| serde_json::from_str::<Vec<Session>>(&s)) {
      Some(Ok(sessions)) => {
        for s in sessions {
          map.insert(s.id.clone(), s);
        }
        tracing::info!("loaded {} session(s) from {}", map.len(), path.display());
      },
      Some(Err(e)) => warn!("could not parse {}: {e}", path.display()),
      None => {},
    }
    Self { inner: Mutex::new(map), path }
  }

  fn persist(&self, map: &BTreeMap<String, Session>) {
    if let Some(parent) = self.path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    let json = match serde_json::to_string_pretty(&map.values().collect::<Vec<_>>()) {
      Ok(j) => j,
      Err(e) => {
        warn!("serialize sessions: {e}");
        return;
      },
    };
    let tmp = self.path.with_extension("json.tmp");
    if std::fs::write(&tmp, json).and_then(|_| std::fs::rename(&tmp, &self.path)).is_err() {
      warn!("could not persist sessions to {}", self.path.display());
    }
  }

  pub fn create(&self) -> Session {
    let s = Session::new();
    self.inner.lock().unwrap().insert(s.id.clone(), s.clone());
    self.persist(&self.inner.lock().unwrap());
    s
  }

  pub fn list(&self) -> Vec<SessionInfo> {
    let map = self.inner.lock().unwrap();
    let mut infos: Vec<SessionInfo> = map.values().map(Session::info).collect();
    infos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));
    infos
  }

  pub fn get(&self, id: &str) -> Option<Session> {
    self.inner.lock().unwrap().get(id).cloned()
  }

  pub fn delete(&self, id: &str) -> bool {
    let removed = self.inner.lock().unwrap().remove(id).is_some();
    if removed {
      self.persist(&self.inner.lock().unwrap());
    }
    removed
  }

  pub fn clear_all(&self) -> usize {
    let mut map = self.inner.lock().unwrap();
    let n = map.len();
    map.clear();
    self.persist(&map);
    n
  }

  /// Appends a message; the first user message names the session.
  pub fn append(&self, id: &str, role: &str, content: &str) -> Option<Session> {
    let mut map = self.inner.lock().unwrap();
    let s = map.get_mut(id)?;
    s.messages.push(ChatMsg { role: role.to_owned(), content: content.to_owned(), ts: now() });
    s.updated_at = now();
    if role == "user" && s.title == "New chat" {
      let mut title: String = content.chars().take(48).collect();
      if title.len() < content.len() {
        title.push('…');
      }
      s.title = title;
    }
    let snapshot = s.clone();
    self.persist(&map);
    Some(snapshot)
  }

  /// Full history as OpenAI-style `{role, content}` JSON values.
  pub fn history_json(&self, id: &str) -> Option<Vec<serde_json::Value>> {
    let map = self.inner.lock().unwrap();
    Some(
      map
        .get(id)?
        .messages
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect(),
    )
  }
}
