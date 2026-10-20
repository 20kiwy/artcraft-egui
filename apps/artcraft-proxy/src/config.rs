//! Hardcoded ZAI plugin connection details, with environment-variable overrides.
//!
//! These constants are the "plugin connect" details wired into the ArtCraft
//! egui rebuild: the app only needs the proxy address (`http://127.0.0.1:8787`)
//! and this proxy handles the ZAI session plumbing.

/// Where the proxy listens (the address hardcoded into the egui app).
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 8787;

/// Z.ai open platform base URL (OpenAI-compatible GLM chat + CogView images).
pub const DEFAULT_ZAI_BASE_URL: &str = "https://api.z.ai/api/paas/v4";

/// Default models used when the app does not request a specific one.
pub const DEFAULT_CHAT_MODEL: &str = "glm-4.6";
pub const DEFAULT_IMAGE_MODEL: &str = "cogview-4";

/// Where chat sessions are persisted (JSON).
pub const DEFAULT_DATA_DIR: &str = "data";

/// The persona appended in front of every ZAI session's history.
pub const SYSTEM_PROMPT: &str = "You are ArtCraft Assistant, the built-in AI copilot of ArtCraft, \
the IDE for artists (rebuilt in pure Rust with egui). You help artists brainstorm ideas, write and \
refine image/video/audio prompts, and give concise art direction. Keep answers tight and practical, \
and when the user asks for an image prompt, return a single polished prompt ready to paste into the \
Image studio.";

#[derive(Clone, Debug)]
pub struct Settings {
  pub host: String,
  pub port: u16,
  pub zai_base_url: String,
  pub api_key: Option<String>,
  pub chat_model: String,
  pub image_model: String,
  pub mock: bool,
  pub data_dir: String,
}

impl Settings {
  /// Hardcoded defaults, overridden by environment variables:
  /// `ZAI_API_KEY`, `ZAI_BASE_URL`, `ZAI_CHAT_MODEL`, `ZAI_IMAGE_MODEL`,
  /// `ARTCRAFT_PROXY_HOST`, `ARTCRAFT_PROXY_PORT`, `ARTCRAFT_PROXY_MOCK`,
  /// `ARTCRAFT_PROXY_DATA_DIR`.
  pub fn from_env() -> Self {
    let api_key = std::env::var("ZAI_API_KEY").ok().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    let force_mock = std::env::var("ARTCRAFT_PROXY_MOCK")
      .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
      .unwrap_or(false);
    let mock = force_mock || api_key.is_none();
    Self {
      host: env_or("ARTCRAFT_PROXY_HOST", DEFAULT_HOST),
      port: env_or("ARTCRAFT_PROXY_PORT", &DEFAULT_PORT.to_string()).parse().unwrap_or(DEFAULT_PORT),
      zai_base_url: env_or("ZAI_BASE_URL", DEFAULT_ZAI_BASE_URL).trim_end_matches('/').to_owned(),
      api_key,
      chat_model: env_or("ZAI_CHAT_MODEL", DEFAULT_CHAT_MODEL),
      image_model: env_or("ZAI_IMAGE_MODEL", DEFAULT_IMAGE_MODEL),
      mock,
      data_dir: env_or("ARTCRAFT_PROXY_DATA_DIR", DEFAULT_DATA_DIR),
    }
  }

  /// `"live"` when a real ZAI key is configured, `"mock"` otherwise.
  pub fn mode(&self) -> &'static str {
    if self.mock { "mock" } else { "live" }
  }
}

fn env_or(key: &str, default: &str) -> String {
  std::env::var(key).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_owned())
}
