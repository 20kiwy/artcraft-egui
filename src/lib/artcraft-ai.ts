/**
 * ZAI capabilities for the hosted bridge, powered by `z-ai-web-dev-sdk`.
 *
 * Backend-only (never import from client components). The sandbox injects
 * the credentials, so the egui desktop app needs NOTHING but the preview
 * URL — no API key, ever. This is what flips the app's "PROXY · MOCK" chip
 * to "PROXY · LIVE".
 */

import ZAI from 'z-ai-web-dev-sdk';

/** The ArtCraft persona (same as the Rust proxy's SYSTEM_PROMPT). */
export const SYSTEM_PROMPT =
  'You are ArtCraft Assistant, the built-in AI copilot of ArtCraft, ' +
  'the IDE for artists (rebuilt in pure Rust with egui). You help artists brainstorm ideas, write and ' +
  'refine image/video/audio prompts, and give concise art direction. Keep answers tight and practical, ' +
  'and when the user asks for an image prompt, return a single polished prompt ready to paste into the ' +
  'Image studio.';

/** Sizes the platform image model supports. */
export const IMAGE_SIZES = [
  '1024x1024',
  '768x1344',
  '864x1152',
  '1344x768',
  '1152x864',
  '1440x720',
  '720x1440',
] as const;

export type ChatMsg = { role: 'system' | 'user' | 'assistant'; content: string };

type ZaiClient = Awaited<ReturnType<typeof ZAI.create>>;

// Cache the client on globalThis so Next.js HMR doesn't recreate it per request.
const globalScope = globalThis as unknown as { __artcraftZai?: Promise<ZaiClient> };

export function getZai(): Promise<ZaiClient> {
  globalScope.__artcraftZai ??= ZAI.create();
  return globalScope.__artcraftZai;
}

/** One GLM chat completion (thinking disabled for snappy assistant replies). */
export async function chatComplete(messages: ChatMsg[]): Promise<string> {
  const zai = await getZai();
  const completion = await zai.chat.completions.create({
    messages,
    thinking: { type: 'disabled' },
  });
  const content = completion.choices[0]?.message?.content;
  if (!content) throw new Error('empty completion from GLM');
  return content;
}

/** One CogView image generation → raw PNG bytes. */
export async function generateImage(prompt: string, size: string): Promise<Buffer> {
  const zai = await getZai();
  const safeSize = (IMAGE_SIZES as readonly string[]).includes(size) ? size : '1024x1024';
  const response = await zai.images.generations.create({ prompt, size: safeSize });
  const base64 = response.data?.[0]?.base64;
  if (!base64) throw new Error('image generation returned no data');
  return Buffer.from(base64, 'base64');
}
