/**
 * Session store for the hosted ArtCraft bridge.
 *
 * Mirrors the artcraft-proxy session API shapes exactly (see
 * apps/artcraft/src/proxy.rs) so the egui app cannot tell the difference:
 *
 *   SessionInfo  { id, title, created_at, updated_at, message_count }
 *   SessionFull  { id, title, created_at, updated_at, messages: [{role, content, ts}] }
 *
 * Persisted to `data/bridge-sessions.json` so sessions survive dev-server
 * restarts.
 */

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

export type StoredMsg = { role: 'user' | 'assistant'; content: string; ts: number };

export type StoredSession = {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  messages: StoredMsg[];
};

const DATA_DIR = path.join(process.cwd(), 'data');
const STORE_FILE = path.join(DATA_DIR, 'bridge-sessions.json');

const globalScope = globalThis as unknown as { __artcraftSessions?: Map<string, StoredSession> };

function store(): Map<string, StoredSession> {
  if (!globalScope.__artcraftSessions) globalScope.__artcraftSessions = new Map();
  return globalScope.__artcraftSessions;
}

const globalLoaded = globalThis as unknown as { __artcraftSessionsLoaded?: Promise<void> };

async function persist(): Promise<void> {
  try {
    await mkdir(DATA_DIR, { recursive: true });
    await writeFile(STORE_FILE, JSON.stringify([...store().values()], null, 2));
  } catch (err) {
    console.error('[artcraft-bridge] session persist failed:', err);
  }
}

/** Load the JSON store once per server process. */
export async function ensureLoaded(): Promise<void> {
  globalLoaded.__artcraftSessionsLoaded ??= (async () => {
    try {
      const raw = await readFile(STORE_FILE, 'utf8');
      for (const session of JSON.parse(raw) as StoredSession[]) store().set(session.id, session);
      console.log(`[artcraft-bridge] loaded ${store().size} session(s) from ${STORE_FILE}`);
    } catch {
      /* first run — no store file yet */
    }
  })();
  await globalLoaded.__artcraftSessionsLoaded;
}

function newId(): string {
  return crypto.randomUUID();
}

function toInfo(s: StoredSession) {
  return {
    id: s.id,
    title: s.title,
    created_at: s.created_at,
    updated_at: s.updated_at,
    message_count: s.messages.length,
  };
}

function truncate(text: string, n: number): string {
  const clean = text.replace(/\s+/g, ' ').trim();
  return clean.length <= n ? clean : `${clean.slice(0, n - 1)}…`;
}

export async function createSession(): Promise<StoredSession> {
  await ensureLoaded();
  const now = Math.floor(Date.now() / 1000);
  const session: StoredSession = { id: newId(), title: 'New chat', created_at: now, updated_at: now, messages: [] };
  store().set(session.id, session);
  await persist();
  return session;
}

export async function listSessions(): Promise<{ sessions: ReturnType<typeof toInfo>[] }> {
  await ensureLoaded();
  const sessions = [...store().values()].sort((a, b) => b.updated_at - a.updated_at).map(toInfo);
  return { sessions };
}

export async function getSession(id: string): Promise<StoredSession | null> {
  await ensureLoaded();
  return store().get(id) ?? null;
}

export async function deleteSession(id: string): Promise<boolean> {
  await ensureLoaded();
  const ok = store().delete(id);
  if (ok) await persist();
  return ok;
}

export async function clearSessions(): Promise<number> {
  await ensureLoaded();
  const n = store().size;
  store().clear();
  await persist();
  return n;
}

/** Append messages, auto-title from the first user message. */
export async function appendMessages(id: string, msgs: StoredMsg[]): Promise<StoredSession | null> {
  await ensureLoaded();
  const session = store().get(id);
  if (!session) return null;
  session.messages.push(...msgs);
  session.updated_at = Math.floor(Date.now() / 1000);
  if (session.title === 'New chat') {
    const firstUser = msgs.find((m) => m.role === 'user');
    if (firstUser) session.title = truncate(firstUser.content, 48) || 'New chat';
  }
  await persist();
  return session;
}
