'use client'

import {
  AppWindow,
  ArrowRight,
  Boxes,
  Cpu,
  FolderGit2,
  Github,
  Image as ImageIcon,
  MessageSquare,
  Network,
  Package,
  Play,
  Server,
  Terminal,
  Wand2,
} from 'lucide-react'

const ENDPOINTS: Array<[string, string]> = [
  ['GET /health', 'liveness + live/mock mode'],
  ['GET /api/v1/config', 'models, sizes, ZAI base URL'],
  ['POST /api/v1/sessions', 'create a ZAI chat session'],
  ['POST /api/v1/sessions/{id}/chat', 'chat on a stored session'],
  ['POST /api/v1/images', 'CogView text-to-image → base64 PNG'],
  ['ANY /api/v1/openai/{*path}', 'raw passthrough to ZAI (streams)'],
]

const PAGES: Array<[string, string, string]> = [
  ['Create › Image', 'prompt box, model/size/count, Enter-to-generate, feed + lightbox', 'Wand2'],
  ['Create › Assistant', 'GLM chat over persisted ZAI sessions', 'MessageSquare'],
  ['Assets › Library', 'auto-saved generations, lazy textures, reveal in Explorer', 'ImageIcon'],
  ['System › Settings', 'proxy URL, connection test, session manager', 'AppWindow'],
]

const CI: Array<[string, string, string]> = [
  ['proxy.yml', 'ubuntu-latest · windows-latest', 'artcraft-proxy-{linux,windows}-x86_64'],
  ['artcraft.yml', 'windows-latest', 'artcraft-egui-windows-x64.zip'],
]

function Hud({ children }: { children: React.ReactNode }) {
  return (
    <span className="font-mono text-[11px] uppercase tracking-[0.15em] text-zinc-500">{children}</span>
  )
}

function Badge({ children }: { children: React.ReactNode }) {
  return (
    <span className="rounded-[3px] border border-amber-300/40 bg-amber-300/10 px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-amber-300">
      {children}
    </span>
  )
}

export default function Home() {
  return (
    <div className="min-h-screen bg-[#121316] text-[#f2f1ee] flex flex-col">
      {/* Title bar */}
      <header className="flex h-10 items-center gap-3 border-b border-white/15 px-4" style={{ background: '#0a0a0c' }}>
        <span className="h-2.5 w-2.5 rounded-[2px] bg-[#2d81ff]" />
        <span className="font-mono text-xs uppercase tracking-[0.15em]">artcraft</span>
        <Badge>egui</Badge>
        <Badge>rust</Badge>
        <span className="ml-auto hidden font-mono text-[11px] uppercase tracking-[0.15em] text-zinc-500 sm:block">
          proxy · live on :8787
        </span>
      </header>

      <main className="mx-auto w-full max-w-5xl flex-1 px-5 py-10 sm:px-8">
        {/* Hero */}
        <section className="text-center">
          <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-lg bg-[#0a0a0c] shadow-[0_0_0_1px_rgba(255,255,255,0.15)]">
            <Cpu className="h-8 w-8 text-[#2d81ff]" aria-hidden />
          </div>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl">ArtCraft — egui rebuild</h1>
          <p className="mx-auto mt-4 max-w-2xl text-base text-zinc-400">
            A full desktop remake of <span className="text-zinc-200">storytold/artcraft</span> in pure Rust with
            egui — <span className="text-zinc-200">no TypeScript, no webview</span> — wired to{' '}
            <span className="text-zinc-200">artcraft-proxy</span>, a Rust bridge that connects the app to ZAI
            sessions (GLM chat + CogView images).
          </p>
          <div className="mt-6 flex flex-wrap items-center justify-center gap-2">
            <Badge>eframe 0.36</Badge>
            <Badge>axum</Badge>
            <Badge>glm-4.6</Badge>
            <Badge>cogview-4</Badge>
            <Badge>mit / apache-2.0</Badge>
          </div>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
            <a
              href="https://github.com/storytold/artcraft"
              target="_blank"
              rel="noreferrer"
              className="inline-flex h-11 items-center gap-2 rounded-[3px] bg-white px-5 text-sm font-semibold text-[#0b0b0c] transition hover:bg-zinc-200"
            >
              <Github className="h-4 w-4" aria-hidden /> Upstream repo
            </a>
            <span className="inline-flex h-11 items-center gap-2 rounded-[3px] border border-white/15 bg-[#1e1f22] px-5 text-sm font-medium text-zinc-200">
              <Terminal className="h-4 w-4" aria-hidden /> <code className="font-mono text-xs">cargo run -p artcraft</code>
            </span>
          </div>
        </section>

        {/* Architecture */}
        <section className="mt-14" aria-labelledby="arch">
          <Hud id="arch">architecture</Hud>
          <div className="mt-4 grid gap-3 md:grid-cols-3">
            {[
              {
                icon: AppWindow,
                title: 'apps/artcraft',
                sub: 'egui desktop app',
                body: 'Brutalist theme mirrored 1:1 from artcraft-native: custom 40px title bar, 232px sidebar, rounded content panel, Inter / Archivo / Geist Mono.',
              },
              {
                icon: Server,
                title: 'apps/artcraft-proxy',
                sub: 'ZAI plugin bridge (axum)',
                body: 'Sessions with history + system prompt, chat completions, CogView images as base64 PNG, OpenAI-compatible passthrough. Mock mode with no key.',
              },
              {
                icon: Network,
                title: 'ZAI (Z.ai)',
                sub: 'glm-4.6 · cogview-4',
                body: 'Hardcoded base URL https://api.z.ai/api/paas/v4 with ZAI_API_KEY env override — set a key and the app goes live.',
              },
            ].map((c) => (
              <article key={c.title} className="rounded-lg border border-white/15 bg-[#101014] p-5">
                <c.icon className="h-5 w-5 text-[#2d81ff]" aria-hidden />
                <h3 className="mt-3 font-mono text-sm font-semibold">{c.title}</h3>
                <p className="font-mono text-[11px] uppercase tracking-wider text-zinc-500">{c.sub}</p>
                <p className="mt-3 text-sm leading-relaxed text-zinc-400">{c.body}</p>
              </article>
            ))}
          </div>
        </section>

        {/* App pages */}
        <section className="mt-12" aria-labelledby="pages">
          <Hud id="pages">inside the app</Hud>
          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            {PAGES.map(([title, body, icon]) => {
              const Icon = icon === 'Wand2' ? Wand2 : icon === 'MessageSquare' ? MessageSquare : icon === 'ImageIcon' ? ImageIcon : AppWindow
              return (
                <article key={title} className="flex gap-4 rounded-lg border border-white/15 bg-[#101014] p-5">
                  <Icon className="mt-0.5 h-5 w-5 shrink-0 text-amber-300" aria-hidden />
                  <div>
                    <h3 className="text-sm font-semibold">{title}</h3>
                    <p className="mt-1 text-sm text-zinc-400">{body}</p>
                  </div>
                </article>
              )
            })}
          </div>
        </section>

        {/* Endpoints + CI */}
        <section className="mt-12 grid gap-6 lg:grid-cols-2" aria-labelledby="api">
          <div>
            <Hud id="api">proxy api</Hud>
            <ul className="mt-4 divide-y divide-white/10 overflow-hidden rounded-lg border border-white/15 bg-[#101014]">
              {ENDPOINTS.map(([route, what]) => (
                <li key={route} className="flex flex-col gap-1 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
                  <code className="font-mono text-xs text-[#74aaff]">{route}</code>
                  <span className="text-xs text-zinc-500">{what}</span>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <Hud>ci — github workflows</Hud>
            <ul className="mt-4 space-y-3">
              {CI.map(([wf, os, artifact]) => (
                <li key={wf} className="rounded-lg border border-white/15 bg-[#101014] p-4">
                  <div className="flex items-center gap-2">
                    <Package className="h-4 w-4 text-emerald-400" aria-hidden />
                    <span className="font-mono text-xs font-semibold">{wf}</span>
                  </div>
                  <p className="mt-2 text-xs text-zinc-400">{os}</p>
                  <p className="mt-1 font-mono text-[11px] text-zinc-500">{artifact}</p>
                </li>
              ))}
            </ul>
          </div>
        </section>

        {/* Repo layout + quickstart */}
        <section className="mt-12 grid gap-6 lg:grid-cols-2" aria-labelledby="repo">
          <div>
            <Hud id="repo">repository</Hud>
            <div className="mt-4 rounded-lg border border-white/15 bg-[#060607] p-5 font-mono text-xs leading-6 text-zinc-400">
              <p><span className="text-zinc-500">├─</span> apps/artcraft <span className="text-zinc-600">· egui app</span></p>
              <p><span className="text-zinc-500">├─</span> apps/artcraft-proxy <span className="text-zinc-600">· axum bridge</span></p>
              <p><span className="text-zinc-500">├─</span> clone/artcraft <span className="text-zinc-600">· submodule</span></p>
              <p><span className="text-zinc-500">├─</span> clone/artcraftx <span className="text-zinc-600">· submodule</span></p>
              <p><span className="text-zinc-500">├─</span> .github/workflows <span className="text-zinc-600">· CI</span></p>
              <p><span className="text-zinc-500">└─</span> README.md</p>
            </div>
          </div>
          <div>
            <Hud>quickstart</Hud>
            <div className="mt-4 space-y-2 rounded-lg border border-white/15 bg-[#060607] p-5 font-mono text-xs leading-6">
              <p className="text-zinc-500"># terminal 1 — the ZAI plugin bridge</p>
              <p className="text-emerald-300">$ cargo run -p artcraft-proxy</p>
              <p className="mt-2 text-zinc-500"># terminal 2 — the desktop app</p>
              <p className="text-emerald-300">$ cargo run -p artcraft</p>
              <p className="mt-2 text-zinc-500"># live AI mode</p>
              <p className="text-[#74aaff]">ZAI_API_KEY=sk-… cargo run -p artcraft-proxy</p>
            </div>
          </div>
        </section>

        {/* Footer nav */}
        <section className="mt-12 flex flex-wrap items-center justify-center gap-6 border-t border-white/10 pt-8 text-sm">
          <a href="https://github.com/storytold/artcraft" target="_blank" rel="noreferrer" className="inline-flex items-center gap-2 text-zinc-400 transition hover:text-zinc-100">
            <FolderGit2 className="h-4 w-4" aria-hidden /> clone/artcraft <ArrowRight className="h-3 w-3" aria-hidden />
          </a>
          <a href="https://github.com/storytold/artcraftx" target="_blank" rel="noreferrer" className="inline-flex items-center gap-2 text-zinc-400 transition hover:text-zinc-100">
            <Boxes className="h-4 w-4" aria-hidden /> clone/artcraftx <ArrowRight className="h-3 w-3" aria-hidden />
          </a>
          <span className="inline-flex items-center gap-2 text-zinc-500">
            <Play className="h-4 w-4" aria-hidden /> mock mode needs no API key
          </span>
        </section>
      </main>

      {/* Sticky footer */}
      <footer className="mt-auto border-t border-white/15 px-5 py-4" style={{ background: '#0a0a0c' }}>
        <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-2">
          <Hud>artcraft egui rebuild · pure rust · no typescript</Hud>
          <Hud>zai sessions · glm + cogview · v0.41.0</Hud>
        </div>
      </footer>
    </div>
  )
}
