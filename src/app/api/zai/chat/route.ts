/**
 * ZAI bridge: chat completions backed by the z-ai-web-dev-sdk (GLM).
 *
 * The Rust `artcraft-proxy` calls this route when it runs without a
 * ZAI_API_KEY (see `--bridge-url`): the SDK authenticates with the platform
 * integration server-side, so the desktop app never handles credentials.
 *
 * Request : { messages: [{ role, content }], model?: string }
 * Response: { content, model, backend: "zai-bridge" }
 */

import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'nodejs';
export const maxDuration = 300;

interface BridgeMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export async function POST(req: NextRequest) {
  try {
    const body = (await req.json()) as { messages?: BridgeMessage[]; model?: string };
    const messages = Array.isArray(body.messages) ? body.messages : [];
    if (messages.length === 0) {
      return NextResponse.json({ error: 'messages must be a non-empty array' }, { status: 400 });
    }

    const ZAI = (await import('z-ai-web-dev-sdk')).default;
    const zai = await ZAI.create();

    // The SDK wants system prompts on the `assistant` role.
    const sdkMessages = messages.map((m) => ({
      role: m.role === 'system' ? ('assistant' as const) : m.role,
      content: String(m.content ?? ''),
    }));

    const completion = await zai.chat.completions.create({
      messages: sdkMessages,
      thinking: { type: 'disabled' },
    });

    const content = completion.choices[0]?.message?.content ?? '';
    if (!content.trim()) {
      return NextResponse.json({ error: 'empty completion from GLM' }, { status: 502 });
    }

    return NextResponse.json({
      content,
      model: body.model ?? 'glm-4.6',
      backend: 'zai-bridge',
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error('[zai-bridge] chat failed:', message);
    return NextResponse.json({ error: `bridge chat failed: ${message}` }, { status: 500 });
  }
}
