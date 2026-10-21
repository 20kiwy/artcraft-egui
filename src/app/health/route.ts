/**
 * Public bridge for the ArtCraft egui desktop app.
 *
 * GET /health → forwards to the local artcraft-proxy (127.0.0.1:8787).
 *
 * This lets the preview URL be pasted into the desktop app as
 * "Settings → Proxy URL": the gateway only exposes port 3000, so the
 * Next.js app relays the proxy endpoints (see src/app/api/v1/[...path]).
 */

export const dynamic = 'force-dynamic';

const PROXY_ORIGIN = process.env.ARTCRAFT_PROXY_ORIGIN ?? 'http://127.0.0.1:8787';

export async function GET() {
  try {
    const upstream = await fetch(`${PROXY_ORIGIN}/health`, { cache: 'no-store' });
    return new Response(upstream.body, {
      status: upstream.status,
      headers: {
        'content-type': upstream.headers.get('content-type') ?? 'application/json',
        'cache-control': 'no-store',
      },
    });
  } catch {
    return Response.json(
      {
        ok: false,
        error: `artcraft-proxy unreachable at ${PROXY_ORIGIN} — start it with: cargo run -p artcraft-proxy`,
        service: 'artcraft-proxy-bridge',
      },
      { status: 502 },
    );
  }
}
