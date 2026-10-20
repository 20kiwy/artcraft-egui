//! HTTP error type that renders as a JSON body.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct HttpError(pub StatusCode, pub String);

impl HttpError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self(StatusCode::BAD_REQUEST, msg.into())
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    Self(StatusCode::NOT_FOUND, msg.into())
  }

  pub fn bad_gateway(msg: impl Into<String>) -> Self {
    Self(StatusCode::BAD_GATEWAY, msg.into())
  }

  pub fn internal(msg: impl Into<String>) -> Self {
    Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into())
  }
}

impl IntoResponse for HttpError {
  fn into_response(self) -> Response {
    (self.0, Json(json!({ "error": self.1 }))).into_response()
  }
}

/// Any error (`anyhow`, reqwest, …) becomes a 502 with its message.
impl<E: std::fmt::Display> From<E> for HttpError {
  fn from(e: E) -> Self {
    Self::internal(e.to_string())
  }
}
