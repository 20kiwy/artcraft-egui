//! HTTP surface of the plugin: health, config, sessions, chat, images and a
//! raw OpenAI-compatible passthrough for power users.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::{Settings, SYSTEM_PROMPT};
use crate::error::HttpError;
use crate::mock;
use crate::sessions::SessionStore;
use crate::zai::ZaiClient;

#[derive(Clone)]
pub struct AppState {
  pub settings: Settings,
  pub zai: ZaiClient,
  pub sessions: Arc<SessionStore>,
}

pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/api/v1/config", get(config))
    .route("/api/v1/sessions", post(create_session).get(list_sessions).delete(clear_sessions))
    .route("/api/v1/sessions/{id}", get(get_session).delete(delete_session))
    .route("/api/v1/sessions/{id}/chat", post(session_chat))
    .route("/api/v1/images", post(generate_image))
    .route("/api/v1/chat/completions", post(chat_completions))
    .route("/api/v1/openai/{*rest}", any(openai_passthrough))
    .fallback(fallback)
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}

// ---------------------------------------------------------------- health ---

async fn health(State(st): State<AppState>) -> Json<Value> {
  Json(json!({
    "ok": true,
    "service": "artcraft-proxy",
    "version": env!("CARGO_PKG_VERSION"),
    "mode": st.settings.mode(),
    "plugin": "zai-sessions",
  }))
}

async fn config(State(st): State<AppState>) -> Json<Value> {
  Json(json!({
    "mode": st.settings.mode(),
    "backend": st.settings.backend(),
    "zai_base_url": st.settings.zai_base_url,
    "api_key_set": st.settings.api_key.is_some(),
    "chat_model": st.settings.chat_model,
    "image_model": st.settings.image_model,
    "chat_models": ["glm-4.6", "glm-4.5", "glm-4.5-air", "glm-4.5-flash"],
    "image_models": ["cogview-4", "cogview-3-flash"],
    "image_sizes": ["1024x1024", "1152x896", "1344x768", "896x1152", "768x1344"],
    "app_default_proxy_url": "http://127.0.0.1:8787",
    "system_prompt": SYSTEM_PROMPT,
  }))
}

// -------------------------------------------------------------- sessions ---

async fn create_session(State(st): State<AppState>) -> Json<Value> {
  let s = st.sessions.create();
  Json(json!({ "id": s.id, "title": s.title, "created_at": s.created_at }))
}

async fn list_sessions(State(st): State<AppState>) -> Json<Value> {
  Json(json!({ "sessions": st.sessions.list() }))
}

async fn clear_sessions(State(st): State<AppState>) -> Json<Value> {
  let n = st.sessions.clear_all();
  Json(json!({ "cleared": n }))
}

async fn get_session(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, HttpError> {
  let s = st.sessions.get(&id).ok_or_else(|| HttpError::not_found("session not found"))?;
  Ok(Json(serde_json::to_value(s).unwrap_or_else(|_| json!({}))))
}

async fn delete_session(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, HttpError> {
  let removed = st.sessions.delete(&id);
  if removed {
    Ok(Json(json!({ "deleted": true, "id": id })))
  } else {
    Err(HttpError::not_found("session not found"))
  }
}

// ------------------------------------------------------------------ chat ---

#[derive(Deserialize)]
struct ChatBody {
  message: String,
  model: Option<String>,
}

/// Turn-based chat on a stored session: history + system prompt go to ZAI,
/// the assistant reply is appended to the session and returned.
async fn session_chat(
  State(st): State<AppState>,
  Path(id): Path<String>,
  Json(body): Json<ChatBody>,
) -> Result<Json<Value>, HttpError> {
  let message = body.message.trim().to_owned();
  if message.is_empty() {
    return Err(HttpError::bad_request("message must not be empty"));
  }
  st.sessions.append(&id, "user", &message).ok_or_else(|| HttpError::not_found("session not found"))?;

  let model = body.model.unwrap_or_else(|| st.settings.chat_model.clone());
  let (reply, usage) = if st.settings.mock {
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    (mock::chat_reply(&message), None)
  } else {
    let mut payload = vec![json!({ "role": "system", "content": SYSTEM_PROMPT })];
    payload.extend(st.sessions.history_json(&id).unwrap_or_default());
    let out = st
      .zai
      .chat_completion(&model, &payload)
      .await
      .map_err(|e| HttpError::bad_gateway(format!("ZAI chat failed: {e}")))?;
    let raw = out["choices"][0]["message"]["content"].clone();
    let reply = match raw.as_str() {
      Some(s) => s.to_owned(),
      None => serde_json::to_string_pretty(&raw).unwrap_or_else(|_| "…".into()),
    };
    (reply, out.get("usage").cloned())
  };

  st
    .sessions
    .append(&id, "assistant", &reply)
    .ok_or_else(|| HttpError::internal("session vanished mid-turn"))?;

  Ok(Json(json!({
    "session_id": id,
    "reply": reply,
    "model": model,
    "mode": st.settings.mode(),
    "usage": usage,
  })))
}

/// Direct OpenAI-compatible completion (no session storage) — handy for tools.
async fn chat_completions(State(st): State<AppState>, Json(mut body): Json<Value>) -> Result<Json<Value>, HttpError> {
  if body.get("model").and_then(Value::as_str).is_none() {
    body["model"] = json!(st.settings.chat_model);
  }
  let messages = body
    .get("messages")
    .and_then(Value::as_array)
    .cloned()
    .ok_or_else(|| HttpError::bad_request("messages array is required"))?;
  if st.settings.mock {
    let last = messages.iter().rev().find_map(|m| m["content"].as_str()).unwrap_or("hello");
    return Ok(Json(json!({
      "choices": [{ "message": { "role": "assistant", "content": mock::chat_reply(last) } }],
      "mode": "mock",
    })));
  }
  let model = body["model"].as_str().unwrap_or_default().to_owned();
  let out = st
    .zai
    .chat_completion(&model, &messages)
    .await
    .map_err(|e| HttpError::bad_gateway(format!("ZAI chat failed: {e}")))?;
  Ok(Json(out))
}

// ---------------------------------------------------------------- images ---

#[derive(Deserialize)]
struct ImageBody {
  prompt: String,
  size: Option<String>,
  model: Option<String>,
}

/// Text-to-image: returns PNG/base64 so the egui app can decode it directly.
async fn generate_image(State(st): State<AppState>, Json(body): Json<ImageBody>) -> Result<Json<Value>, HttpError> {
  let prompt = body.prompt.trim().to_owned();
  if prompt.is_empty() {
    return Err(HttpError::bad_request("prompt must not be empty"));
  }
  let size = body.size.unwrap_or_else(|| "1024x1024".into());
  let model = body.model.unwrap_or_else(|| st.settings.image_model.clone());

  let (bytes, mime) = if st.settings.mock {
    let bytes = mock::image_png(&prompt, &size).map_err(|e| HttpError::internal(format!("mock render failed: {e}")))?;
    (bytes, "image/png".to_owned())
  } else {
    st
      .zai
      .generate_image(&model, &prompt, &size)
      .await
      .map_err(|e| HttpError::bad_gateway(format!("ZAI image failed: {e}")))?
  };

  Ok(Json(json!({
    "image_base64": base64::engine::general_purpose::STANDARD.encode(&bytes),
    "mime": mime,
    "prompt": prompt,
    "model": model,
    "size": size,
    "mode": st.settings.mode(),
  })))
}

// ----------------------------------------------------------- passthrough ---

/// Raw passthrough: `/api/v1/openai/<upstream path>` → `{ZAI_BASE_URL}/<path>`
/// with the plugin key attached. Streaming responses stream through.
async fn openai_passthrough(
  State(st): State<AppState>,
  Path(rest): Path<String>,
  axum::extract::RawQuery(query): axum::extract::RawQuery,
  method: axum::http::Method,
  headers: HeaderMap,
  body: Bytes,
) -> Response {
  let _ = headers; // auth + content-type are set by the client below
  let qs = query.map(|q| format!("?{q}")).unwrap_or_default();
  let upstream = format!("/{rest}{qs}");
  match st.zai.passthrough(method, &upstream, body).await {
    Ok(resp) => {
      let status = resp.status();
      let mut builder = Response::builder().status(status);
      if let Some(ct) = resp.headers().get(header::CONTENT_TYPE) {
        builder = builder.header(header::CONTENT_TYPE, ct.clone());
      }
      let stream = resp.bytes_stream();
      builder.body(axum::body::Body::from_stream(stream)).unwrap_or_else(|e| {
        HttpError::internal(format!("stream error: {e}")).into_response()
      })
    },
    Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("upstream failed: {e}") }))).into_response(),
  }
}

async fn fallback(uri: axum::http::Uri) -> impl IntoResponse {
  (
    StatusCode::NOT_FOUND,
    Json(json!({
      "error": format!("no route for {} — try /health or /api/v1/config", uri.path()),
    })),
  )
}
