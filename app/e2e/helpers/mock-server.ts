// oxlint-disable no-console
import { type Server, createServer } from 'node:http';

// Fixed port so specs can build a deterministic instance URL. The app reaches
// it via `127.0.0.1` on desktop, and via an `adb reverse` tunnel on Android
// (set up in app-launcher.ts) so the same URL works on every platform.
export const MOCK_PORT = 4500;
// Host must be `localhost` (not the `127.0.0.1` literal): the Android network
// security config (gen/android .../network_security_config.xml) only permits
// cleartext HTTP for the `localhost` domain, so the webview `fetch` in the
// setup form is blocked for `127.0.0.1`. `localhost` resolves to 127.0.0.1 on
// desktop and tunnels through `adb reverse` on the emulator.
export const MOCK_URL = `http://localhost:${MOCK_PORT}`;

// Mirrors the contract the app checks for in `setup` onsubmit and the Rust
// connection check: `/api/health` must return 200 with an `X-Api-Version`
// header for the instance to be considered a valid, reachable Positron server.
const API_VERSION = 'e2e-mock';

// The fake user the mock "logs in". The uuid must be a valid UUID because the
// Rust side deserializes it into `uuid::Uuid`.
export const MOCK_USER = {
  uuid: '11111111-1111-1111-1111-111111111111',
  name: 'E2E Tester',
  email: 'e2e@example.com'
};

// The token value the exchange endpoint hands back via the `centaurus_jwt`
// cookie. PKCE is intentionally NOT verified here — the mock trusts any code.
const MOCK_TOKEN = 'mock-jwt-token';

let server: Server | undefined = undefined;

export const startMockServer = async (): Promise<void> =>
  new Promise((resolve, reject) => {
    server = createServer((req, res) => {
      const method = req.method ?? 'GET';
      const url = (req.url ?? '').split('?')[0];
      const route = `${method} ${url}`;

      // Reachability / setup health check.
      if (url === '/api/health') {
        res.writeHead(200, {
          'Content-Type': 'application/json',
          'X-Api-Version': API_VERSION
        });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // PKCE code exchange (triggered by the `positron://auth` deep link). The
      // verifier is ignored; we just mint a session cookie.
      if (route === 'POST /api/auth/app/exchange') {
        res.writeHead(200, {
          'Content-Type': 'application/json',
          'Set-Cookie': `centaurus_jwt=${MOCK_TOKEN}; Path=/; HttpOnly`
        });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // Login-code approval (triggered from the `/login` page confirm button).
      if (route === 'POST /api/auth/app/approve') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // Token validity probe run on startup when a token is present.
      if (route === 'GET /api/auth/test_token') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ valid: true, exp_short: false }));
        return;
      }

      if (route === 'GET /api/auth/refresh_token') {
        res.writeHead(200, {
          'Content-Type': 'application/json',
          'Set-Cookie': `centaurus_jwt=${MOCK_TOKEN}; Path=/; HttpOnly`
        });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // Current user info loaded after authentication.
      if (route === 'GET /api/user/info') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(MOCK_USER));
        return;
      }

      // User avatar bytes. The content is irrelevant to the specs; only a
      // 200 with a body is required so `load_user_info` succeeds.
      if (url.startsWith('/api/user/info/avatar/')) {
        res.writeHead(200, { 'Content-Type': 'image/webp' });
        res.end(Buffer.from([0x52, 0x49, 0x46, 0x46]));
        return;
      }

      res.writeHead(404, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'not found' }));
    });

    server.on('error', reject);
    server.listen(MOCK_PORT, '127.0.0.1', () => {
      console.log(`Mock backend listening on ${MOCK_URL}`);
      resolve();
    });
  });

export const stopMockServer = async (): Promise<void> =>
  new Promise((resolve) => {
    if (!server) {
      resolve();
      return;
    }
    server.close(() => {
      console.log('Mock backend stopped');
      server = undefined;
      resolve();
    });
  });
