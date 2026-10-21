/**
 * Spawns the Rust `artcraft-proxy` binary as a child of the Next.js server
 * and keeps it alive (checked every 5 s).
 *
 * This is what makes the preview URL usable as the egui desktop app's proxy
 * URL, because the sandbox gateway only exposes port 3000:
 *
 *   egui app → https://<preview-url>/{health,api/v1/*}   (gateway)
 *     → Next.js bridge routes (src/app/health, src/app/api/v1/[...path])
 *       → artcraft-proxy (this child process, 127.0.0.1:8787)
 *         → ZAI sessions (GLM chat + CogView images)
 *
 * The sandbox reaps stray background processes, so a separately started
 * proxy would die between sessions — as a child of the long-lived Next.js
 * server it survives, and the watchdog here respawns it if it ever exits.
 * The port check before every spawn keeps repeated `register()` calls
 * (dev-server restarts) from creating duplicates.
 */

import { spawn } from 'node:child_process';
import { existsSync, openSync } from 'node:fs';
import net from 'node:net';
import path from 'node:path';

const root = process.cwd();
const bin = ['target/release/artcraft-proxy', 'target/debug/artcraft-proxy']
  .map((p) => path.join(root, p))
  .find((p) => existsSync(p));

const port = Number(process.env.ARTCRAFT_PROXY_PORT ?? 8787);

if (!bin) {
  console.warn('[artcraft-bridge] no artcraft-proxy binary — build with: cargo build -p artcraft-proxy');
} else {
  const logFd = openSync(path.join(root, 'proxy-bridge.log'), 'a');

  const portInUse = () =>
    new Promise<boolean>((resolve) => {
      const sock = net.connect({ host: '127.0.0.1', port }, () => {
        sock.destroy();
        resolve(true);
      });
      sock.once('error', () => resolve(false));
      sock.setTimeout(500, () => {
        sock.destroy();
        resolve(false);
      });
    });

  const ensureProxy = async () => {
    try {
      if (await portInUse()) return;
      const child = spawn(
        bin,
        ['--host', '127.0.0.1', '--port', String(port), '--data-dir', path.join(root, 'apps/artcraft-proxy/data')],
        { stdio: ['ignore', logFd, logFd], env: process.env },
      );
      child.on('error', (err) => console.error('[artcraft-bridge] spawn failed:', err.message));
      child.on('exit', (code, signal) =>
        console.warn(`[artcraft-bridge] artcraft-proxy exited (code=${code}, signal=${signal})`),
      );
      console.log(`[artcraft-bridge] spawned artcraft-proxy (pid ${child.pid}) on 127.0.0.1:${port}`);
    } catch (err) {
      console.error('[artcraft-bridge] ensureProxy failed:', err);
    }
  };

  void ensureProxy();
  const watchdog = setInterval(ensureProxy, 5000);
  // Never keep the server alive just for the watchdog.
  watchdog.unref?.();
}
