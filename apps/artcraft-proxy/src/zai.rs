//! Minimal ZAI (Z.ai / GLM) HTTP client — the plugin side of the bridge.
//!
//! Everything here talks to the OpenAI-compatible endpoints documented by
//! the Z.ai open platform: `POST {base}/chat/completions` and
//! `POST {base}/images/generations`.

use anyhow::{bail, Context, Result};
use base64::Engine;
use serde_json::{json, Value};

use crate::config::Settings;

#[derive(Clone)]
pub struct ZaiClient {
  http: reqwest::Client,
  base: String,
  key: Option<String>,
}

impl ZaiClient {
  pub fn new(settings: &Settings) -> Self {
    let http = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(180))
      .user_agent(concat!("artcraft-proxy/", env!("CARGO_PKG_VERSION")))
      .build()
      .expect("reqwest client");
    Self { http, base: settings.zai_base_url.clone(), key: settings.api_key.clone() }
  }

  /// Chat completion → the full upstream JSON (choices[0].message.content).
  pub async fn chat_completion(&self, model: &str, messages: &[Value]) -> Result<Value> {
    let key = self.key.as_deref().context("ZAI_API_KEY is not configured")?;
    let url = format!("{}/chat/completions", self.base);
    let resp = self
      .http
      .post(url)
      .bearer_auth(key)
      .json(&json!({ "model": model, "messages": messages, "stream": false }))
      .send()
      .await
      .context("ZAI chat request failed")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("ZAI chat response was not JSON")?;
    if !status.is_success() {
      let msg = body["error"]["message"].as_str().unwrap_or("ZAI rejected the chat request");
      bail!("ZAI chat error {status}: {msg}");
    }
    Ok(body)
  }

  /// Image generation → raw image bytes + mime type (URLs are downloaded here
  /// so the desktop app only ever sees base64 data).
  pub async fn generate_image(&self, model: &str, prompt: &str, size: &str) -> Result<(Vec<u8>, String)> {
    let key = self.key.as_deref().context("ZAI_API_KEY is not configured")?;
    let url = format!("{}/images/generations", self.base);
    let resp = self
      .http
      .post(url)
      .bearer_auth(key)
      .json(&json!({ "model": model, "prompt": prompt, "size": size }))
      .send()
      .await
      .context("ZAI image request failed")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("ZAI image response was not JSON")?;
    if !status.is_success() {
      let msg = body["error"]["message"].as_str().unwrap_or("ZAI rejected the image request");
      bail!("ZAI image error {status}: {msg}");
    }
    let item = body["data"]
      .get(0)
      .ok_or_else(|| anyhow::anyhow!("ZAI image response contained no data"))?
      .clone();

    if let Some(b64) = item["b64_json"].as_str() {
      let bytes = base64::engine::general_purpose::STANDARD.decode(b64).context("decode ZAI b64 image")?;
      return Ok((bytes, "image/png".into()));
    }
    if let Some(url) = item["url"].as_str() {
      let dl = self.http.get(url).send().await.context("download generated image")?;
      let mime = dl
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .split(';')
        .next()
        .unwrap_or("image/png")
        .to_owned();
      let bytes = dl.bytes().await.context("read generated image bytes")?;
      return Ok((bytes.to_vec(), mime));
    }
    bail!("ZAI image response had neither b64_json nor url");
  }

  /// Raw passthrough for power users: `{method} {base}{path}` with the plugin
  /// key attached. Streams the upstream response straight back.
  pub async fn passthrough(
    &self,
    method: reqwest::Method,
    path_and_query: &str,
    body: axum::body::Bytes,
  ) -> Result<reqwest::Response> {
    let url = format!("{}{}", self.base, path_and_query);
    let mut req = self.http.request(method, url).header(reqwest::header::CONTENT_TYPE, "application/json");
    if let Some(key) = self.key.as_deref() {
      req = req.bearer_auth(key);
    }
    Ok(req.body(body).send().await?)
  }
}
