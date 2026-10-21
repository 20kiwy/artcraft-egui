//! Hardcoded ZAI plugin connection details, with CLI-flag and environment overrides.
//!
//! These constants are the "plugin connect" details wired into the ArtCraft
//! egui rebuild: the app only needs the proxy address (`http://127.0.0.1:8787`)
//! and this proxy handles the ZAI session plumbing.

/// Where the proxy listens by default.
///
/// `127.0.0.1` (loopback) instead of `0.0.0.0`: the egui app always connects
/// from the same machine, and a wildcard bind triggers Windows Firewall /
/// antivirus blocks (os error 10013) for no benefit. Use `--host 0.0.0.0`
/// to expose the proxy on your LAN.
pub const DEFAULT_HOST: &str = "127.0.0.1";
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

pub const HELP_TEXT: &str = "\
artcraft-proxy — ArtCraft AI plugin bridge (ZAI sessions -> local HTTP)

The ArtCraft egui app expects this proxy at http://127.0.0.1:8787 (change it in
the app: Settings -> Proxy URL). Live modes, in order of preference:
  1. ZAI_API_KEY set        -> calls api.z.ai directly
  2. --bridge-url set       -> a server-side ZAI bridge answers with real
     GLM/CogView models and holds the credentials (the desktop app and the
     proxy never see a key)
With neither, the proxy runs in MOCK mode (canned replies + placeholder
images) so you can test the app offline.

Usage:
  artcraft-proxy [options]

Options:
  --host <addr>        Address to bind          (default: 127.0.0.1)
  --port <number>      Port to bind            (default: 8787)
  --data-dir <dir>     Session storage folder  (default: data)
  --bridge-url <url>   Server-side ZAI bridge base URL (live mode without a key)
  -h, --help           Show this help and exit

Environment overrides:
  ZAI_API_KEY             Z.ai API key -> live mode
  ZAI_BASE_URL            default: https://api.z.ai/api/paas/v4
  ZAI_CHAT_MODEL          default: glm-4.6
  ZAI_IMAGE_MODEL         default: cogview-4
  ARTCRAFT_BRIDGE_URL     same as --bridge-url
  ARTCRAFT_PROXY_HOST     same as --host
  ARTCRAFT_PROXY_PORT     same as --port
  ARTCRAFT_PROXY_DATA_DIR same as --data-dir
  ARTCRAFT_PROXY_MOCK     force mock mode (1/true/yes/on)

Windows notes:
  If the port is blocked (os error 10013) it is usually inside a Hyper-V /
  WinNAT reserved range. Check:  netsh interface ipv4 show excludedportrange protocol=tcp
  Then just pick a free port:    artcraft-proxy.exe --port 8899
";

/// Parsed command-line flags (highest precedence, above env vars).
#[derive(Clone, Debug, Default)]
pub struct CliArgs {
  pub host: Option<String>,
  pub port: Option<u16>,
  pub data_dir: Option<String>,
  pub bridge_url: Option<String>,
}

impl CliArgs {
  /// Hand-rolled parser; returns `Err(HELP_TEXT)` for `--help` and
  /// `Err(message)` for unknown/invalid flags.
  pub fn parse() -> Result<Self, String> {
    let mut args = Self::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
      // Accept both `--flag value` and `--flag=value`.
      let (key, inline) = match arg.split_once('=') {
        Some((k, v)) => (k.to_owned(), Some(v.to_owned())),
        None => (arg.clone(), None),
      };
      match key.as_str() {
        "-h" | "--help" => return Err(HELP_TEXT.to_owned()),
        "--host" => args.host = Some(take_value(&mut iter, inline, "--host")?),
        "--port" => {
          let raw = take_value(&mut iter, inline, "--port")?;
          args.port = Some(raw.parse().map_err(|_| {
            format!("invalid value for --port: {raw:?} (expected 0-65555, e.g. --port 8899)")
          })?);
        }
        "--data-dir" => args.data_dir = Some(take_value(&mut iter, inline, "--data-dir")?),
        "--bridge-url" => args.bridge_url = Some(take_value(&mut iter, inline, "--bridge-url")?),
        other => {
          return Err(format!(
            "unknown argument: {other}\n\n{HELP_TEXT}"
          ));
        }
      }
    }
    Ok(args)
  }
}

fn take_value(
  iter: &mut impl Iterator<Item = String>,
  inline: Option<String>,
  flag: &str,
) -> Result<String, String> {
  inline.or_else(|| iter.next()).ok_or_else(|| format!("missing value for {flag} (e.g. {flag} 8899)"))
}

#[derive(Clone, Debug)]
pub struct Settings {
  pub host: String,
  pub port: u16,
  pub zai_base_url: String,
  pub api_key: Option<String>,
  /// Server-side ZAI bridge (e.g. the sandbox's Next.js routes): real GLM /
  /// CogView answers without this proxy holding any credential.
  pub bridge_url: Option<String>,
  pub chat_model: String,
  pub image_model: String,
  pub mock: bool,
  pub data_dir: String,
}

impl Settings {
  /// Hardcoded defaults, overridden by environment variables:
  /// `ZAI_API_KEY`, `ZAI_BASE_URL`, `ZAI_CHAT_MODEL`, `ZAI_IMAGE_MODEL`,
  /// `ARTCRAFT_PROXY_HOST`, `ARTCRAFT_PROXY_PORT`, `ARTCRAFT_PROXY_MOCK`,
  /// `ARTCRAFT_PROXY_DATA_DIR` — and finally by CLI flags via
  /// [`Settings::apply_cli`].
  pub fn from_env() -> Self {
    let api_key = std::env::var("ZAI_API_KEY").ok().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    let bridge_url = std::env::var("ARTCRAFT_BRIDGE_URL")
      .ok()
      .map(|s| s.trim().trim_end_matches('/').to_owned())
      .filter(|s| !s.is_empty());
    let force_mock = std::env::var("ARTCRAFT_PROXY_MOCK")
      .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
      .unwrap_or(false);
    let mock = force_mock || (api_key.is_none() && bridge_url.is_none());
    Self {
      host: env_or("ARTCRAFT_PROXY_HOST", DEFAULT_HOST),
      port: env_or("ARTCRAFT_PROXY_PORT", &DEFAULT_PORT.to_string()).parse().unwrap_or(DEFAULT_PORT),
      zai_base_url: env_or("ZAI_BASE_URL", DEFAULT_ZAI_BASE_URL).trim_end_matches('/').to_owned(),
      api_key,
      bridge_url,
      chat_model: env_or("ZAI_CHAT_MODEL", DEFAULT_CHAT_MODEL),
      image_model: env_or("ZAI_IMAGE_MODEL", DEFAULT_IMAGE_MODEL),
      mock,
      data_dir: env_or("ARTCRAFT_PROXY_DATA_DIR", DEFAULT_DATA_DIR),
    }
  }

  /// CLI flags win over environment variables.
  pub fn apply_cli(&mut self, cli: &CliArgs) {
    if let Some(h) = &cli.host {
      self.host = h.clone();
    }
    if let Some(p) = cli.port {
      self.port = p;
    }
    if let Some(d) = &cli.data_dir {
      self.data_dir = d.clone();
    }
    if let Some(b) = &cli.bridge_url {
      self.bridge_url = Some(b.trim().trim_end_matches('/').to_owned()).filter(|s| !s.is_empty());
    }
    // Re-evaluate after CLI overrides: a bridge flag upgrades mock to live.
    if self.api_key.is_none() && self.bridge_url.is_some() {
      self.mock = false;
    }
  }

  /// `"live"` when real models answer (ZAI key or server-side bridge),
  /// `"mock"` otherwise.
  pub fn mode(&self) -> &'static str {
    if self.mock { "mock" } else { "live" }
  }

  /// How requests reach the models (for the config endpoint and logs).
  pub fn backend(&self) -> &'static str {
    if self.api_key.is_some() {
      "zai-direct"
    } else if self.bridge_url.is_some() {
      "zai-bridge"
    } else {
      "mock"
    }
  }
}

fn env_or(key: &str, default: &str) -> String {
  std::env::var(key).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_owned())
}
