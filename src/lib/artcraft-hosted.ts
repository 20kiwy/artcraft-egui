/**
 * Hosted ArtCraft API — the ZAI plugin backend that runs entirely inside the
 * Next.js server.
 *
 * Implements the exact `artcraft-proxy` API contract the egui app speaks
 * (see apps/artcraft/src/proxy.rs), but powers chat with the platform GLM
 * and images with the platform image model via z-ai-web-dev-sdk. Credentials
 * never leave the server: the desktop app only needs the preview URL.
 *
 * `handleHostedApi` returns `null` for paths it does not own (e.g. the raw
 * `/api/v1/openai/*` passthrough) so the caller can fall back to relaying to
 * the Rust artcraft-proxy child on 127.0.0.1:8787.
 */

import { IMAGE_SIZES, SYSTEM_PROMPT, chatComplete, generateImage, type ChatMsg } from './artcraft-ai';
import { appendMessages, clearSessions, createSession, deleteSession, getSession, listSessions } from './artcraft-sessions';

export const CHAT_MODEL = 'glm-4.6';
export const IMAGE_MODEL = 'cogview-4';
export const HOSTED_VERSION = '0.41.0-hosted';

export function hostedHealth() {
  return {
    ok: true,
    service: 'artcraft-proxy',
    version: HOSTED_VERSION,
    mode: 'live' as const,
    backend: 'z-ai-web-dev-sdk (server-side credentials — no API key needed in the app)',
  };
}

export function hostedConfig() {
  return {
    mode: 'live' as const,
    zai_base_url: 'https://api.z.ai/api/paas/v4',
    api_key_set: true,
    chat_model: CHAT_MODEL,
    image_model: IMAGE_MODEL,
    chat_models: [CHAT_MODEL],
    image_models: [IMAGE_MODEL],
    image_sizes: [...IMAGE_SIZES],
    system_prompt: SYSTEM_PROMPT,
    backend: 'z-ai-web-dev-sdk (hosted)',
  };
}

const json = (body: unknown, status = 200) =>
  Response.json(body, { status, headers: { 'cache-control': 'no-store' } });

/** POST /api/v1/sessions/{id}/chat → ChatReply */
async function chat(sessionId: string, body: { message?: string }): Promise<Response> {
  const message = (body.message ?? '').trim();
  if (!message) return json({ error: 'message is required' }, 400);

  const existing = await getSession(sessionId);
  if (!existing) return json({ error: 'session not found' }, 404);

  const now = Math.floor(Date.now() / 1000);
  const history: ChatMsg[] = existing.messages.map((m) => ({
    role: m.role === 'assistant' ? 'assistant' : 'user',
    content: m.content,
  }));

  const reply = await chatComplete([
    { role: 'system', content: SYSTEM_PROMPT },
    ...history,
    { role: 'user', content: message },
  ]);

  await appendMessages(sessionId, [
    { role: 'user', content: message, ts: now },
    { role: 'assistant', content: reply, ts: Math.floor(Date.now() / 1000) },
  ]);

  return json({ session_id: sessionId, reply, model: CHAT_MODEL, mode: 'live' });
}

/** POST /api/v1/images → ImageOut */
async function image(body: { prompt?: string; size?: string; model?: string }): Promise<Response> {
  const prompt = (body.prompt ?? '').trim();
  if (!prompt) return json({ error: 'prompt is required' }, 400);
  const size = body.size && (IMAGE_SIZES as readonly string[]).includes(body.size) ? body.size : '1024x1024';

  const bytes = await generateImage(prompt, size);
  // Sniff the real container (the platform model usually returns JPEG).
  const mime = bytes[0] === 0xff && bytes[1] === 0xd8 ? 'image/jpeg' : 'image/png';
  return json({
    image_base64: bytes.toString('base64'),
    mime,
    prompt,
    model: IMAGE_MODEL,
    size,
    mode: 'live',
  });
}

/**
 * Handles a request if it belongs to the hosted API; returns `null` to let
 * the caller fall through to the Rust-proxy relay.
 */
export async function handleHostedApi(method: string, segments: string[], body: unknown): Promise<Response | null> {
  try {
    // /api/v1/config
    if (method === 'GET' && segments[0] === 'config' && segments.length === 1) {
      return json(hostedConfig());
    }

    // /api/v1/sessions (list / create / clear)
    if (segments[0] === 'sessions' && segments.length === 1) {
      if (method === 'GET') return json(await listSessions());
      if (method === 'POST') return json(await createSession());
      if (method === 'DELETE') return json({ cleared: await clearSessions() });
      return null;
    }

    // /api/v1/sessions/{id}[...]
    if (segments[0] === 'sessions' && segments.length >= 2) {
      const id = segments[1];
      if (segments.length === 2) {
        if (method === 'GET') {
          const session = await getSession(id);
          return session ? json(session) : json({ error: 'session not found' }, 404);
        }
        if (method === 'DELETE') {
          const ok = await deleteSession(id);
          return ok ? json({ deleted: true, id }) : json({ error: 'session not found' }, 404);
        }
        return null;
      }
      // /api/v1/sessions/{id}/chat
      if (segments.length === 3 && segments[2] === 'chat' && method === 'POST') {
        return await chat(id, (body ?? {}) as { message?: string });
      }
      return null;
    }

    // /api/v1/images
    if (method === 'POST' && segments[0] === 'images' && segments.length === 1) {
      return await image((body ?? {}) as { prompt?: string; size?: string });
    }

    // Anything else (e.g. /api/v1/openai/*) → fall through to the Rust proxy.
    return null;
  } catch (err) {
    console.error('[artcraft-bridge] hosted api error:', err);
    return json({ error: `hosted ZAI backend failed: ${err instanceof Error ? err.message : String(err)}` }, 502);
  }
}
