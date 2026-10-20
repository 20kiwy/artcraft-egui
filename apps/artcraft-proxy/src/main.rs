//! artcraft-proxy: the ArtCraft AI plugin bridge.
//!
//! A pure-Rust HTTP service that connects the egui-rebuilt ArtCraft desktop app
//! to ZAI sessions (GLM chat completions + CogView image generation).
//! The ZAI plugin connection details are hardcoded here (see [`config`]) with
//! environment-variable overrides, so the desktop app can just point at this
//! proxy and use AI out of the box.

mod config;
mod error;
mod mock;
mod routes;
mod sessions;
mod zai;

use std::sync::Arc;

use sessions::SessionStore;
use zai::ZaiClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "artcraft_proxy=info,tower_http=warn".into()),
    )
    .init();

  let settings = config::Settings::from_env();
  let store = Arc::new(SessionStore::load(std::path::Path::new(&settings.data_dir)));
  let state = routes::AppState { settings: settings.clone(), zai: ZaiClient::new(&settings), sessions: store };
  let app = routes::router(state);

  let addr = format!("{}:{}", settings.host, settings.port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!(
    "artcraft-proxy v{} listening on http://{addr} — ZAI plugin mode: {} (chat: {}, image: {})",
    env!("CARGO_PKG_VERSION"),
    settings.mode(),
    settings.chat_model,
    settings.image_model,
  );
  if settings.mock {
    tracing::warn!("no ZAI_API_KEY set — running in MOCK mode (canned replies + generated placeholder images)");
  }

  axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
  Ok(())
}

async fn shutdown_signal() {
  let _ = tokio::signal::ctrl_c().await;
  tracing::info!("shutdown signal received");
}
