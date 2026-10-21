# ArtCraft — egui rebuild (pure Rust, no TypeScript)

A full desktop **remake of [Storytold ArtCraft](https://github.com/storytold/artcraft) in [egui](https://github.com/emilk/egui)** — zero TypeScript, zero webview — plus a pure-Rust **`artcraft-proxy`** that connects the app to **ZAI sessions** (GLM chat + CogView image generation) so AI works out of the box.

```
┌─────────────────────┐   HTTP    ┌─────────────────────┐   HTTPS   ┌──────────────────┐
│  apps/artcraft      │ ────────► │  apps/artcraft-proxy│ ────────► │  ZAI (Z.ai)      │
│  egui desktop app   │ hardcoded │  axum plugin bridge │  Bearer   │  glm-4.6 (chat)  │
│  (no TypeScript)    │ 127.0.0.1 │  sessions + mock    │  API key  │  cogview-4 (img) │
└─────────────────────┘   :8787   └─────────────────────┘           └──────────────────┘
```

The upstream sources are vendored as git submodules for reference:

- `clone/artcraft` — upstream ArtCraft (Tauri 2 + `artcraft-native` egui client + TS frontend) — MIT OR Apache-2.0
- `clone/artcraftx` — companion pared-down client — unlicensed, reference only

The egui rebuild mirrors `artcraft-native`'s brutalist theme 1:1 (near-black surfaces, `#2d81ff` brand accent, Inter/Archivo/Geist Mono variable fonts, 3 px control corners, 40 px custom title bar + 232 px sidebar + rounded content panel).

## Repository layout

```
.
├── Cargo.toml               # workspace: apps/artcraft + apps/artcraft-proxy
├── apps/
│   ├── artcraft/            # the egui desktop app (bin: artcraft)
│   │   └── src/             #   app, shell, theme, pages (home/image/assistant/library/settings),
│   │                        #   titlebar, lightbox, icons, widgets, proxy client, state
│   └── artcraft-proxy/      # the ZAI plugin bridge (bin: artcraft-proxy)
│       └── src/             #   axum routes, ZAI client, session store, mock engine
├── clone/                   # upstream repos as git submodules (reference only)
├── .github/workflows/
│   └── build.yml            # one run → proxy (Linux + Windows) + egui app (Windows)
└── README.md
```

## Quickstart

```bash
# 1. start the ZAI plugin bridge (defaults to http://127.0.0.1:8787)
cargo run -p artcraft-proxy

# 2. run the desktop app (another terminal)
cargo run -p artcraft
```

Without configuration the proxy runs in **mock mode** (canned assistant replies + procedurally generated placeholder art), so everything is testable with no account. For real AI:

```bash
ZAI_API_KEY=sk-... cargo run -p artcraft-proxy     # live mode: GLM + CogView
```

### Hardcoded plugin connection details

The egui app ships pointing at the proxy; the proxy ships pointing at ZAI:

| Where | Constant | Value |
|---|---|---|
| `apps/artcraft/src/state.rs` | `DEFAULT_PROXY_URL` | `http://127.0.0.1:8787` |
| `apps/artcraft-proxy/src/config.rs` | `DEFAULT_ZAI_BASE_URL` | `https://api.z.ai/api/paas/v4` |
| 〃 | `DEFAULT_CHAT_MODEL` | `glm-4.6` |
| 〃 | `DEFAULT_IMAGE_MODEL` | `cogview-4` |

Every value has an environment override (`ZAI_API_KEY`, `ZAI_BASE_URL`, `ZAI_CHAT_MODEL`, `ZAI_IMAGE_MODEL`, `ARTCRAFT_PROXY_HOST`, `ARTCRAFT_PROXY_PORT`, `ARTCRAFT_PROXY_MOCK`, `ARTCRAFT_PROXY_DATA_DIR`), and the binary takes CLI flags on top:

```text
artcraft-proxy [--host <addr>] [--port <n>] [--data-dir <dir>] [-h | --help]

# examples
artcraft-proxy.exe --port 8899          # run on another port (update the app: Settings → Proxy URL)
artcraft-proxy.exe --host 0.0.0.0       # expose on your LAN instead of loopback
artcraft-proxy.exe --help               # full help with env vars + Windows notes
```

### Windows: "os error 10013" when starting the proxy

`10013` (WSAEACCES) means Windows refused the bind — almost always because the port landed inside a **Hyper-V / WinNAT reserved (excluded) range**, or a firewall/antivirus blocked the exe:

```bat
:: 1. see if your port is inside a reserved range
netsh interface ipv4 show excludedportrange protocol=tcp

:: 2. easiest fix — pick a port outside every reserved range
artcraft-proxy-windows-x86_64.exe --port 8899

:: 3. or release the dynamic reservations (Terminal as Administrator; restarts WinNAT)
net stop winnat && net start winnat

:: 4. port already used by another app (error 10048)?
netstat -ano | findstr :8787
```

The proxy binds `127.0.0.1` by default (no firewall prompt, all the egui app needs). If you run it on a non-default port, point the app at it via **Settings → Proxy URL**.

### Hosting the proxy behind the sandbox preview URL

The hosted sandbox only exposes port 3000, so the Next.js app bridges the proxy for you:

- `src/instrumentation.ts` + `src/lib/artcraft-proxy-child.ts` spawn `artcraft-proxy` as a **child of the Next.js server** (watchdog respawns it every 5 s if it exits).
- `src/app/health/route.ts` and `src/app/api/v1/[...path]/route.ts` relay `/health` + `/api/v1/*` to `127.0.0.1:8787` (method/query/body preserved, streaming pass-through).

Result: paste the **preview URL** (the HTTPS URL from the preview panel / "Open in New Tab", no trailing slash needed) into the egui app's **Settings → Proxy URL** and AI just works — **no API key in the app, ever**. The hosted bridge (`src/lib/artcraft-hosted.ts`) serves the same artcraft-proxy API contract but powers it with `z-ai-web-dev-sdk` server-side (GLM chat + image generation), so the app's titlebar chip flips to **PROXY · LIVE**. Sessions persist to `data/bridge-sessions.json`; `/api/v1/openai/*` still falls through to the Rust proxy child (`ARTCRAFT_PROXY_ORIGIN` overrides it). A demo art session — every prompt + artwork compiled into one document — ships at `public/artcraft-art-session.pdf` (preview URL → `/artcraft-art-session.pdf`), with the raw pieces in `public/artcraft-gallery/`.

## The proxy API

| Route | Purpose |
|---|---|
| `GET /health` | liveness + mode (`live`/`mock`) |
| `GET /api/v1/config` | models, sizes, ZAI base URL, key presence |
| `POST /api/v1/sessions` | create a ZAI chat session |
| `GET /api/v1/sessions` | list sessions (auto-titled from first message) |
| `GET/DELETE /api/v1/sessions/{id}` | fetch history / delete |
| `POST /api/v1/sessions/{id}/chat` | chat on a session (history + system prompt sent every turn) |
| `POST /api/v1/images` | text-to-image → base64 PNG (CogView) |
| `POST /api/v1/chat/completions` | direct OpenAI-compatible completion |
| `ANY /api/v1/openai/{*path}` | raw passthrough to `{ZAI_BASE_URL}/{path}` (streams) |

Sessions persist to `apps/artcraft-proxy/data/sessions.json`; the egui app stores settings + the image library under `artcraft-data/`.

## The egui app

- **Home** — plugin status, quickstart, upstream info
- **Create › Image** — prompt box with model/size/count pickers, Enter-to-generate, results feed, lightbox, images auto-saved to `artcraft-data/generated/`
- **Create › Assistant** — GLM chat over proxy sessions (create/switch/delete/clear)
- **Assets › Library** — every generated image on disk, lazy textures, reveal in Explorer
- **System › Settings** — proxy URL (hardcoded default pre-filled), connection test, ZAI readout, session manager

Custom frameless window with hand-painted title bar (drag, double-click maximize, min/max/close), hand-painted icon set, per-frame event polling of worker threads — no blocking on the UI thread.

## Build & CI

GitHub Actions builds release binaries on every push/PR (and attaches them to tag releases). One workflow → **all three artifacts in a single run summary**:

| Workflow | Targets | Artifacts |
|---|---|---|
| `.github/workflows/build.yml` | `ubuntu-latest` + `windows-latest` ×2 jobs | `artcraft-proxy-linux-x86_64`, `artcraft-proxy-windows-x86_64`, `artcraft-egui-windows-x64` |

Build locally:

```bash
cargo build --release -p artcraft-proxy   # linux + windows
cargo build --release -p artcraft         # any desktop platform
```

## Security note

A GitHub token was shared in the chat that produced this repository. **Revoke it** (GitHub → Settings → Developer settings → Personal access tokens) — no secret is stored in this repo, and CI needs none (public submodules + the default `GITHUB_TOKEN`).

## License & attribution

- This workspace: MIT OR Apache-2.0.
- Theme, fonts and layout adapted from upstream `artcraft-native` (MIT OR Apache-2.0, © ArtCraft Team).
- Fonts: Inter, Archivo, Geist Mono — SIL Open Font License (see `apps/artcraft/assets/fonts/OFL-*.txt`).
- Upstream trademarks/logos are not used; the app icon is generated at runtime.
