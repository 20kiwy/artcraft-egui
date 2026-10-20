//! artcraft-proxy: the ArtCraft AI plugin bridge.
//!
//! A pure-Rust HTTP service that connects the egui-rebuilt ArtCraft desktop app
//! to ZAI sessions (GLM chat completions + CogView image generation).
//! The ZAI plugin connection details are hardcoded here (see [`config`]) with
//! environment-variable and CLI-flag overrides, so the desktop app can just
//! point at this proxy and use AI out of the box.

mod config;
mod error;
mod mock;
mod routes;
mod sessions;
mod zai;

use std::sync::Arc;

use config::CliArgs;
use sessions::SessionStore;
use zai::ZaiClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // CLI flags first so `--help` works without touching the logger.
  let cli = match CliArgs::parse() {
    Ok(cli) => cli,
    // Help text or usage error: print and exit (0 for help, 2 for bad usage).
    Err(help_or_error) => {
      let is_help = help_or_error.starts_with("artcraft-proxy —");
      println!("{help_or_error}");
      std::process::exit(if is_help { 0 } else { 2 });
    }
  };

  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "artcraft_proxy=info,tower_http=warn".into()),
    )
    .init();

  let mut settings = config::Settings::from_env();
  settings.apply_cli(&cli);
  let store = Arc::new(SessionStore::load(std::path::Path::new(&settings.data_dir)));
  let state = routes::AppState { settings: settings.clone(), zai: ZaiClient::new(&settings), sessions: store };
  let app = routes::router(state);

  let addr = format!("{}:{}", settings.host, settings.port);
  let listener = match tokio::net::TcpListener::bind(&addr).await {
    Ok(listener) => listener,
    Err(err) => return Err(bind_diagnostics(&addr, &err)),
  };
  let bound = listener.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| addr.clone());
  tracing::info!(
    "artcraft-proxy v{} listening on http://{bound} — ZAI plugin mode: {} (chat: {}, image: {})",
    env!("CARGO_PKG_VERSION"),
    settings.mode(),
    settings.chat_model,
    settings.image_model,
  );
  if settings.mock {
    tracing::warn!("no ZAI_API_KEY set — running in MOCK mode (canned replies + generated placeholder images)");
  }
  if bound != format!("127.0.0.1:{}", config::DEFAULT_PORT) {
    tracing::info!(
      "non-default address — point the egui app at it via Settings -> Proxy URL (e.g. http://{bound})"
    );
  }

  axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
  Ok(())
}

/// Turns a bind failure into an actionable error message, with Windows-specific
/// guidance for WSAEACCES (os error 10013) and WSAEADDRINUSE (os error 10048).
fn bind_diagnostics(addr: &str, err: &std::io::Error) -> anyhow::Error {
  let mut msg = format!("failed to bind http://{addr}: {err}");
  let windows_hint = |msg: &mut String| {
    msg.push_str(&format!(
      "\n\nTo run on a different port:\n    artcraft-proxy.exe --port 8899\n\
       (then set the proxy URL in the egui app: Settings -> Proxy URL -> http://127.0.0.1:8899)"
    ));
  };
  match (err.kind(), err.raw_os_error()) {
    // 10013 WSAEACCES — usually a Hyper-V/WinNAT reserved port range, a
    // firewall/AV block on wildcard binds, or a privileged port.
    (std::io::ErrorKind::PermissionDenied, _) | (_, Some(10013)) => {
      msg.push_str(
        "\n\nWindows: this is WSAEACCES (os error 10013). The port is most likely inside a \
         reserved/excluded range (Hyper-V / WinNAT reserve chunks of ports), or a firewall / \
         antivirus is blocking the bind.",
      );
      if cfg!(windows) {
        msg.push_str(
          "\n\nCheck reserved ranges:\n    netsh interface ipv4 show excludedportrange protocol=tcp\n\
           Release dynamic reservations (run Terminal as Administrator, restarts WinNAT):\n    net stop winnat && net start winnat\n\
           If your antivirus flags the exe, allow it or bind loopback only: --host 127.0.0.1",
        );
      } else {
        msg.push_str("\n\nLinux/macOS: ports below 1024 need privileges, or a firewall rule is blocking the bind.");
      }
      windows_hint(&mut msg);
    }
    // 10048 WSAEADDRINUSE / EADDRINUSE — something already listens there.
    (std::io::ErrorKind::AddrInUse, _) | (_, Some(10048)) => {
      msg.push_str("\n\nThe port is already taken by another program.");
      if cfg!(windows) {
        msg.push_str("\nFind the process:\n    netstat -ano | findstr :8787\n    tasklist | findstr <PID>");
      } else {
        msg.push_str("\nFind the process:\n    lsof -i :8787");
      }
      windows_hint(&mut msg);
    }
    _ => {
      msg.push_str("\n\nHint: try a different port with --port <number>.");
      windows_hint(&mut msg);
    }
  }
  anyhow::anyhow!(msg)
}

async fn shutdown_signal() {
  let _ = tokio::signal::ctrl_c().await;
  tracing::info!("shutdown signal received");
}
