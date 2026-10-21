/**
 * Next.js instrumentation hook — runs once when the server boots.
 *
 * Node-only work lives in `lib/artcraft-proxy-child` and is imported behind
 * the `NEXT_RUNTIME === 'nodejs'` guard (the documented pattern — Turbopack
 * tree-shakes it out of the Edge bundle).
 *
 * See `src/lib/artcraft-proxy-child.ts` for what this actually starts.
 */

export async function register() {
  if (process.env.NEXT_RUNTIME === 'nodejs') {
    await import('./lib/artcraft-proxy-child');
  }
}
