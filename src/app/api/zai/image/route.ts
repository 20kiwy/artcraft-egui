/**
 * ZAI bridge: image generation backed by the z-ai-web-dev-sdk (CogView).
 *
 * The Rust `artcraft-proxy` calls this route when it runs without a
 * ZAI_API_KEY (see `--bridge-url`): the SDK authenticates with the platform
 * integration server-side, so the desktop app never handles credentials.
 *
 * Request : { prompt: string, size?: string, model?: string }
 * Response: { image_base64, mime, backend: "zai-bridge" }
 */

import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'nodejs';
export const maxDuration = 300;

const SUPPORTED_SIZES = new Set([
  '1024x1024',
  '768x1344',
  '864x1152',
  '1344x768',
  '1152x864',
  '1440x720',
  '720x1440',
]);

export async function POST(req: NextRequest) {
  try {
    const body = (await req.json()) as { prompt?: string; size?: string; model?: string };
    const prompt = (body.prompt ?? '').trim();
    if (!prompt) {
      return NextResponse.json({ error: 'prompt is required' }, { status: 400 });
    }
    const size = SUPPORTED_SIZES.has(body.size ?? '') ? (body.size as string) : '1024x1024';

    const ZAI = (await import('z-ai-web-dev-sdk')).default;
    const zai = await ZAI.create();

    const response = await zai.images.generations.create({ prompt, size });
    const base64 = response.data?.[0]?.base64;
    if (!base64) {
      return NextResponse.json({ error: 'no image in CogView response' }, { status: 502 });
    }

    return NextResponse.json({
      image_base64: base64,
      mime: 'image/png',
      size,
      backend: 'zai-bridge',
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error('[zai-bridge] image failed:', message);
    return NextResponse.json({ error: `bridge image failed: ${message}` }, { status: 500 });
  }
}
