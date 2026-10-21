/**
 * Catch-all bridge: preview URL → hosted ZAI API (or local artcraft-proxy).
 *
 * The egui desktop app speaks the artcraft-proxy API contract. Requests under
 * /api/v1/* are served in priority order:
 *
 *   1. Hosted backend (src/lib/artcraft-hosted.ts) — GLM chat + CogView-style
 *      image generation via z-ai-web-dev-sdk with server-side credentials.
 *      This is the zero-config path: paste the preview URL in the app and AI
 *      just works, no API key.
 *
 *   2. Rust artcraft-proxy child (127.0.0.1:8787, see
 *      src/lib/artcraft-proxy-child.ts) — fallback for anything the hosted
 *      backend does not own, e.g. the raw /api/v1/openai/* passthrough.
 *
 * Method, query string and body are preserved; responses stream back.
 */

import { NextRequest } from 'next/server';

import { handleHostedApi } from '@/lib/artcraft-hosted';

export const dynamic = 'force-dynamic';

const PROXY_ORIGIN = process.env.ARTCRAFT_PROXY_ORIGIN ?? 'http://127.0.0.1:8787';

/** Headers that must not be relayed between hops. */
const HOP_BY_HOP = new Set([
  'host',
  'connection',
  'keep-alive',
  'transfer-encoding',
  'upgrade',
  'proxy-authenticate',
  'proxy-authorization',
  'te',
  'trailer',
]);

function relayHeaders(headers: Headers): Headers {
  const out = new Headers();
  headers.forEach((value, key) => {
    if (!HOP_BY_HOP.has(key.toLowerCase())) out.set(key, value);
  });
  return out;
}

async function forward(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  const { path = [] } = await ctx.params;

  // --- 1. hosted ZAI backend (server-side credentials) ---
  let parsedBody: unknown = null;
  if (!['GET', 'HEAD'].includes(req.method)) {
    try {
      parsedBody = await req.json();
    } catch {
      parsedBody = null;
    }
  }
  const hosted = await handleHostedApi(req.method, path, parsedBody);
  if (hosted) return hosted;

  // --- 2. fall back to the Rust artcraft-proxy child ---
  const suffix = path.map(encodeURIComponent).join('/');
  const target = `${PROXY_ORIGIN}/api/v1${suffix ? `/${suffix}` : ''}${req.nextUrl.search}`;

  const hasBody = !['GET', 'HEAD'].includes(req.method);
  const body = hasBody ? await req.arrayBuffer() : undefined;

  try {
    const upstream = await fetch(target, {
      method: req.method,
      headers: relayHeaders(req.headers),
      body: body && body.byteLength > 0 ? body : undefined,
      cache: 'no-store',
    });

    const headers = relayHeaders(upstream.headers);
    headers.set('cache-control', 'no-store');
    return new Response(upstream.body, { status: upstream.status, headers });
  } catch {
    return Response.json(
      {
        ok: false,
        error: `no backend for /api/v1/${path.join('/')} — hosted backend declined and artcraft-proxy is unreachable at ${PROXY_ORIGIN}`,
      },
      { status: 502 },
    );
  }
}

export async function GET(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  return forward(req, ctx);
}
export async function POST(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  return forward(req, ctx);
}
export async function DELETE(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  return forward(req, ctx);
}
export async function PUT(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  return forward(req, ctx);
}
export async function PATCH(req: NextRequest, ctx: { params: Promise<{ path?: string[] }> }) {
  return forward(req, ctx);
}
