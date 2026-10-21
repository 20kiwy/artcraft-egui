/**
 * Catch-all bridge: preview URL → local artcraft-proxy.
 *
 * Forwards every method under /api/v1/* to the artcraft-proxy listening on
 * 127.0.0.1:8787, preserving method, headers, query string and body, and
 * streams the upstream response back (works for the OpenAI-compatible
 * streaming passthrough at /api/v1/openai/* too).
 *
 * With this route the egui desktop app can use the public preview URL as its
 * proxy URL (Settings → Proxy URL), because the sandbox gateway only exposes
 * port 3000.
 */

import { NextRequest } from 'next/server';

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

  // Rebuild the upstream path from the decoded catch-all segments.
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
        error: `artcraft-proxy unreachable at ${PROXY_ORIGIN} — start it with: cargo run -p artcraft-proxy`,
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
