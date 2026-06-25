// oxlint-disable no-console
import { type IncomingMessage, type Server, createServer } from 'node:http';

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
  email: 'e2e@example.com',
  name: 'E2E Tester',
  uuid: '11111111-1111-1111-1111-111111111111'
};

// The token value the exchange endpoint hands back via the `centaurus_jwt`
// cookie. PKCE is intentionally NOT verified here — the mock trusts any code.
const MOCK_TOKEN = 'mock-jwt-token';

// The authorization code the browser login page hands back to the app through
// the `positron://auth` deep link. The exchange endpoint ignores its value.
export const MOCK_AUTH_CODE = 'mock-auth-code';

/**
 * What the mock observed during a test, so specs (which run in Node and can
 * reach the server directly) can assert that the app drove the real flow:
 * which PKCE challenge the browser login page received, and what the native
 * client POSTed to the exchange/approve endpoints. Reset between tests via
 * `POST /__test__/reset`.
 */
interface MockState {
  authChallenge: string | undefined;
  authStartedAt: number | undefined;
  exchange: unknown;
  exchangeCount: number;
  approve: unknown;
  approveCount: number;
}

const freshState = (): MockState => ({
  approve: undefined,
  approveCount: 0,
  authChallenge: undefined,
  authStartedAt: undefined,
  exchange: undefined,
  exchangeCount: 0
});

let state: MockState = freshState();

/**
 * A note as the backend's `/api/notes/management` endpoints return it. The shape
 * must match the Rust `NoteInfo` struct exactly — `list_notes` deserializes the
 * array into `Vec<NoteInfo>`, so a missing field (notably `last_updated`, which
 * has no serde default) makes the whole list fetch fail. Every note is owned by
 * the mock user so the owner-only UI (delete, share, transfer) is reachable.
 */
interface MockNote {
  id: string;
  title: string;
  preview: string;
  owner: { id: string; name: string };
  shared_with: never[];
  public_access: null;
  is_owner: boolean;
  can_edit: boolean;
  last_updated: string;
}

let notes = new Map<string, MockNote>();
// `undefined` means unlimited; set via `POST /__test__/config` for limit tests.
let noteMaxPerUser: number | undefined = undefined;

const resetNotes = (): void => {
  notes = new Map();
  noteMaxPerUser = undefined;
};

const makeNote = (title: string): MockNote => ({
  can_edit: true,
  id: globalThis.crypto.randomUUID(),
  is_owner: true,
  last_updated: '2026-06-25T12:00:00',
  owner: { id: MOCK_USER.uuid, name: MOCK_USER.name },
  preview: '',
  // oxlint-disable-next-line no-null
  public_access: null,
  shared_with: [],
  title
});

let server: Server | undefined = undefined;

/**
 * HTML served at `/auth/app`, the page the app opens in the system browser when
 * the user presses "Login". It stands in for the real Positron web login: it
 * reads the PKCE `challenge` the app passed, and — exactly like the real server
 * after a successful browser sign-in — bounces straight back into the app via
 * the `positron://auth` deep link, carrying an authorization code. The app then
 * exchanges that code for a session, so pressing one button drives the entire
 * end-to-end auth handshake.
 */
const loginPageHtml = (code: string): string => `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Mock Positron Login</title>
  </head>
  <body>
    <p id="status">Signing you in…</p>
    <script>
      (function () {
        var params = new URLSearchParams(window.location.search);
        var challenge = params.get('challenge');
        if (!challenge) {
          document.getElementById('status').textContent = 'missing challenge';
          return;
        }
        // Hand control back to the native app through the custom-scheme deep
        // link, the same way the real login page does once auth succeeds.
        window.location.href =
          'positron://auth?code=' + encodeURIComponent(${JSON.stringify(code)});
      })();
    </script>
  </body>
</html>
`;

const readBody = async (req: IncomingMessage): Promise<unknown> => {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(chunk as Buffer);
  }
  if (chunks.length === 0) {
    return undefined;
  }
  try {
    return JSON.parse(Buffer.concat(chunks).toString('utf8'));
  } catch {
    return undefined;
  }
};

export const startMockServer = async (): Promise<void> =>
  new Promise((resolve, reject) => {
    // oxlint-disable-next-line complexity
    server = createServer((req, res) => {
      const method = req.method ?? 'GET';
      const fullUrl = req.url ?? '';
      const [url] = fullUrl.split('?');
      const query = new URLSearchParams(fullUrl.split('?')[1] ?? '');
      const route = `${method} ${url}`;

      // --- Test introspection (host-only; never hit by the app) -------------
      if (route === 'GET /__test__/state') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(state));
        return;
      }

      if (route === 'POST /__test__/reset') {
        state = freshState();
        resetNotes();
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // Sets the per-user note quota so limit handling (409 / disabled Create)
      // can be exercised. Host-only; never hit by the app.
      if (route === 'POST /__test__/config') {
        readBody(req)
          .then((body) => {
            noteMaxPerUser = (body as { max_per_user?: number })?.max_per_user;
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ status: 'ok' }));
          })
          .catch(() => {
            res.writeHead(500, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'read failed' }));
          });
        return;
      }

      // Reachability / setup health check.
      if (url === '/api/health') {
        res.writeHead(200, {
          'Content-Type': 'application/json',
          'X-Api-Version': API_VERSION
        });
        res.end(JSON.stringify({ status: 'ok' }));
        return;
      }

      // Browser login page opened by the app's "Login" button via `openUrl`.
      // Records the PKCE challenge and bounces back into the app by deep link.
      if (route === 'GET /auth/app') {
        state.authChallenge = query.get('challenge') ?? undefined;
        state.authStartedAt = Date.now();
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end(loginPageHtml(MOCK_AUTH_CODE));
        return;
      }

      // PKCE code exchange (triggered by the `positron://auth` deep link). The
      // verifier is ignored; we just mint a session cookie.
      if (route === 'POST /api/auth/app/exchange') {
        readBody(req)
          .then((body) => {
            state.exchange = body;
            state.exchangeCount += 1;
            res.writeHead(200, {
              'Content-Type': 'application/json',
              'Set-Cookie': `centaurus_jwt=${MOCK_TOKEN}; Path=/; HttpOnly`
            });
            res.end(JSON.stringify({ status: 'ok' }));
          })
          .catch(() => {
            res.writeHead(500, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'read failed' }));
          });
        return;
      }

      // Login-code approval (triggered from the `/login` page confirm button).
      if (route === 'POST /api/auth/app/approve') {
        readBody(req)
          .then((body) => {
            state.approve = body;
            state.approveCount += 1;
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ status: 'ok' }));
          })
          .catch(() => {
            res.writeHead(500, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'read failed' }));
          });
        return;
      }

      // Token validity probe run on startup when a token is present.
      if (route === 'GET /api/auth/test_token') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ exp_short: false, valid: true }));
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

      // --- Notes management -------------------------------------------------
      // Note quota; `{}` (no limit) by default. Checked before the `/{id}` match.
      if (route === 'GET /api/notes/management/config') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(
          JSON.stringify(
            noteMaxPerUser === undefined ? {} : { max_per_user: noteMaxPerUser }
          )
        );
        return;
      }

      // Users a note can be shared with. Empty for the single-user mock.
      if (route === 'GET /api/notes/management/users') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify([]));
        return;
      }

      if (url === '/api/notes/management') {
        if (method === 'GET') {
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify([...notes.values()]));
          return;
        }
        if (method === 'POST') {
          readBody(req)
            .then((body) => {
              if (
                noteMaxPerUser !== undefined &&
                notes.size >= noteMaxPerUser
              ) {
                res.writeHead(409, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: 'limit' }));
                return;
              }
              const note = makeNote((body as { title?: string })?.title ?? '');
              notes.set(note.id, note);
              res.writeHead(200, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify(note));
            })
            .catch(() => {
              res.writeHead(500, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify({ error: 'read failed' }));
            });
          return;
        }
        if (method === 'PUT') {
          readBody(req)
            .then((body) => {
              const { note_id, title } = (body ?? {}) as {
                note_id?: string;
                title?: string;
              };
              const note = note_id ? notes.get(note_id) : undefined;
              if (note && title !== undefined) {
                note.title = title;
              }
              res.writeHead(200, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify({ status: 'ok' }));
            })
            .catch(() => {
              res.writeHead(500, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify({ error: 'read failed' }));
            });
          return;
        }
        if (method === 'DELETE') {
          readBody(req)
            .then((body) => {
              const { note_id } = (body ?? {}) as { note_id?: string };
              if (note_id) {
                notes.delete(note_id);
              }
              res.writeHead(200, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify({ status: 'ok' }));
            })
            .catch(() => {
              res.writeHead(500, { 'Content-Type': 'application/json' });
              res.end(JSON.stringify({ error: 'read failed' }));
            });
          return;
        }
      }

      // Single note (loaded by the editor page after create / on open).
      if (method === 'GET' && url.startsWith('/api/notes/management/')) {
        const id = url.slice('/api/notes/management/'.length);
        const note = notes.get(id);
        if (note) {
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify(note));
          return;
        }
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'not found' }));
        return;
      }

      // Snapshots list — empty; the editor tolerates an empty history.
      if (method === 'GET' && url.startsWith('/api/notes/snapshots/')) {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify([]));
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
