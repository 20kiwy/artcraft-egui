/**
 * Public bridge for the ArtCraft egui desktop app.
 *
 * GET /health → reports the hosted ZAI backend (live mode, server-side
 * credentials — the egui app only ever needs the preview URL). This is what
 * flips the app's titlebar chip from "PROXY · MOCK" to "PROXY · LIVE".
 */

import { hostedHealth } from '@/lib/artcraft-hosted';

export const dynamic = 'force-dynamic';

export async function GET() {
  return Response.json(hostedHealth(), { headers: { 'cache-control': 'no-store' } });
}
