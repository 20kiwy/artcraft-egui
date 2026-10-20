//! Blocking HTTP client for the `artcraft-proxy` ZAI plugin. Every call runs
//! on a worker thread (see `state::spawn_event`) so the UI never stalls.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct Health {
  pub ok: bool,
  pub service: String,
  pub version: String,
  pub mode: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigInfo {
  pub mode: String,
  pub zai_base_url: String,
  pub api_key_set: bool,
  pub chat_model: String,
  pub image_model: String,
  pub chat_models: Vec<String>,
  pub image_models: Vec<String>,
  pub image_sizes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
  pub id: String,
  pub title: String,
  pub created_at: u64,
  pub updated_at: u64,
  pub message_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SChatMsg {
  pub role: String,
  pub content: String,
  pub ts: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionFull {
  pub id: String,
  pub title: String,
  pub created_at: u64,
  pub updated_at: u64,
  pub messages: Vec<SChatMsg>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatReply {
  pub session_id: String,
  pub reply: String,
  pub model: String,
  pub mode: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct ImageOut {
  pub image_base64: String,
  pub mime: String,
  pub prompt: String,
  pub model: String,
  pub size: String,
  pub mode: String,
}

pub struct ProxyClient {
  base: String,
  http: reqwest::blocking::Client,
}

impl ProxyClient {
  pub fn new(base: &str) -> Self {
    let http = reqwest::blocking::Client::builder()
      .timeout(std::time::Duration::from_secs(300))
      .user_agent(concat!("artcraft-egui/", env!("CARGO_PKG_VERSION")))
      .build()
      .expect("blocking reqwest client");
    Self { base: base.trim_end_matches('/').to_owned(), http }
  }

  fn url(&self, path: &str) -> String {
    format!("{}{path}", self.base)
  }

  fn checked(&self, resp: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
    let status = resp.status();
    if status.is_success() {
      Ok(resp)
    } else {
      let text = resp.text().unwrap_or_default();
      let msg = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v["error"].as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("HTTP {status}"));
      bail!("proxy error: {msg}");
    }
  }

  pub fn health(&self) -> Result<Health> {
    self.checked(self.http.get(self.url("/health")).send()?)?.json().context("health json")
  }

  pub fn config(&self) -> Result<ConfigInfo> {
    self.checked(self.http.get(self.url("/api/v1/config")).send()?)?.json().context("config json")
  }

  pub fn sessions(&self) -> Result<Vec<SessionInfo>> {
    let v: serde_json::Value = self.checked(self.http.get(self.url("/api/v1/sessions")).send()?)?.json()?;
    Ok(serde_json::from_value(v["sessions"].clone()).unwrap_or_default())
  }

  pub fn create_session(&self) -> Result<String> {
    let v: serde_json::Value = self.checked(self.http.post(self.url("/api/v1/sessions")).send()?)?.json()?;
    Ok(v["id"].as_str().unwrap_or_default().to_owned())
  }

  pub fn session(&self, id: &str) -> Result<SessionFull> {
    self.checked(self.http.get(self.url(&format!("/api/v1/sessions/{id}"))).send()?)?.json().context("session json")
  }

  pub fn delete_session(&self, id: &str) -> Result<()> {
    self.checked(self.http.delete(self.url(&format!("/api/v1/sessions/{id}"))).send()?)?;
    Ok(())
  }

  pub fn clear_sessions(&self) -> Result<usize> {
    let v: serde_json::Value = self.checked(self.http.delete(self.url("/api/v1/sessions")).send()?)?.json()?;
    Ok(v["cleared"].as_u64().unwrap_or(0) as usize)
  }

  pub fn chat(&self, session_id: &str, message: &str, model: Option<&str>) -> Result<ChatReply> {
    let mut body = serde_json::json!({ "message": message });
    if let Some(m) = model {
      body["model"] = serde_json::json!(m);
    }
    self.checked(self.http.post(self.url(&format!("/api/v1/sessions/{session_id}/chat"))).json(&body).send()?)?
      .json()
      .context("chat json")
  }

  pub fn image(&self, prompt: &str, size: &str, model: Option<&str>) -> Result<ImageOut> {
    let mut body = serde_json::json!({ "prompt": prompt, "size": size });
    if let Some(m) = model {
      body["model"] = serde_json::json!(m);
    }
    self.checked(self.http.post(self.url("/api/v1/images")).json(&body).send()?)?.json().context("image json")
  }
}
