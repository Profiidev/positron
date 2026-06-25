import { $, browser } from '@wdio/globals';
import { MOCK_AUTH_CODE, MOCK_URL } from './mock-server.js';
import { openDeepLink, restartAndroidApp } from './app-launcher.js';

export const WEBDRIVER_PORT = 4445;
export const BASE_URL = 'tauri://localhost';

/** What the mock backend observed during the current test. */
export interface MockState {
  authChallenge?: string;
  authStartedAt?: number;
  exchange?: { code?: string; verifier?: string; application?: string };
  exchangeCount: number;
  approve?: { code?: string };
  approveCount: number;
}

/**
 * Reads the mock backend's observation log. Specs run in Node, so they can
 * reach the mock directly (the app talks to it over the adb tunnel) to assert
 * what the app actually sent — e.g. the PKCE challenge the browser page saw.
 */
export const getMockState = async (): Promise<MockState> => {
  const res = await fetch(`${MOCK_URL}/__test__/state`);
  return res.json() as Promise<MockState>;
};

/** Clears the mock backend's observation log. Call in `beforeEach`. */
export const resetMockState = async (): Promise<void> => {
  await fetch(`${MOCK_URL}/__test__/reset`, { method: 'POST' }).catch(() => {});
};

/** Sets the mock backend's per-user note quota (undefined = unlimited). */
export const setNoteLimit = async (max: number): Promise<void> => {
  await fetch(`${MOCK_URL}/__test__/config`, {
    body: JSON.stringify({ max_per_user: max }),
    headers: { 'Content-Type': 'application/json' },
    method: 'POST'
  }).catch(() => {});
};

/**
 * Waits until the system browser has actually fetched the mock login page,
 * returning the PKCE challenge it carried. Proves the app's "Login" button drove
 * the real `openUrl` to the right instance URL with a freshly minted challenge.
 */
export const waitForMockAuthStart = async (
  timeout = 15_000
): Promise<string> => {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    // eslint-disable-next-line no-await-in-loop -- sequential polling is intentional
    const state = await getMockState();
    if (state.authChallenge) {
      return state.authChallenge;
    }
    // eslint-disable-next-line no-await-in-loop -- sequential polling is intentional
    await new Promise((done) => setTimeout(done, 300));
  }
  throw new Error('Browser never requested the mock login page');
};

export const isMobile = (): boolean => {
  const platform = process.env.TAURI_TEST_PLATFORM;
  return platform === 'android' || platform === 'ios';
};

/**
 * Calls a Tauri command through the IPC bridge that the runtime injects into
 * every webview (`window.__TAURI_INTERNALS__`). This lets specs drive and reset
 * the Rust-side app state directly, without going through the UI.
 */
export const invokeCommand = async <T = unknown>(
  command: string,
  args: Record<string, unknown> = {}
): Promise<T> => {
  const result = await browser.execute(
    async (cmd: string, payload: Record<string, unknown>) => {
      const internals = (
        window as unknown as {
          __TAURI_INTERNALS__?: {
            invoke: (c: string, a?: unknown) => Promise<unknown>;
          };
        }
      ).__TAURI_INTERNALS__;

      if (!internals?.invoke) {
        return { __error: 'Tauri IPC bridge not available' };
      }

      try {
        const value = await internals.invoke(cmd, payload);
        return { __ok: value };
      } catch (error: unknown) {
        return { __error: String(error) };
      }
    },
    command,
    args
  );

  const typed = result as { __ok?: T; __error?: string };
  if (typed.__error !== undefined) {
    throw new Error(`invoke(${command}) failed: ${typed.__error}`);
  }
  return typed.__ok as T;
};

/** Current client-side route, e.g. `/`, `/setup`, `/auth`. */
export const getRoute = async (): Promise<string> =>
  browser.execute(() => window.location.pathname);

/** Waits until SvelteKit has hydrated content into the mount point. */
export const waitForAppReady = async (timeout = 15_000): Promise<void> => {
  await browser.waitUntil(
    async () =>
      browser.execute(
        () =>
          document.readyState === 'complete' &&
          (document.querySelector('body > div')?.childNodes.length ?? 0) > 0
      ),
    { timeout, timeoutMsg: 'App did not become ready' }
  );
};

/** Hard-reloads the webview and waits for the SPA to come back up. */
export const reloadApp = async (): Promise<void> => {
  await browser.execute(() => window.location.reload());
  // Let the webview tear down before polling for the fresh document.
  await browser.pause(500);
  await waitForAppReady();
};

export const waitForRoute = async (
  route: string,
  timeout = 10_000
): Promise<void> => {
  await browser.waitUntil(async () => (await getRoute()) === route, {
    timeout,
    timeoutMsg: `Expected route "${route}" but was "${await getRoute()}"`
  });
};

/**
 * Like {@link waitForRoute}, but tolerates the webview being unreachable while
 * polling. During the browser-driven login the system browser comes to the
 * foreground and the app's webview is backgrounded, so `browser.execute` throws
 * transiently; we swallow those and keep polling until the deep link brings the
 * app back and it settles on `route`. Uses a longer default timeout to cover
 * the external browser round-trip.
 */
export const waitForRouteThroughBackground = async (
  route: string,
  timeout = 30_000
): Promise<void> => {
  await browser.waitUntil(
    async () => {
      try {
        return (await getRoute()) === route;
      } catch {
        // Webview backgrounded behind the browser; try again next tick.
        return false;
      }
    },
    {
      timeout,
      timeoutMsg: `Expected route "${route}" after browser login round-trip`
    }
  );
};

/**
 * Fully closes the app and the system browser and relaunches the app as a cold
 * process, then rebinds WebDriver to it with a fresh session. On desktop (no
 * separate process/browser to manage) this degrades to a hard webview reload.
 * Gives every test a brand-new process with no backgrounded state or leftover
 * browser tab carried over from the previous one.
 */
export const restartApp = async (): Promise<void> => {
  if (!isMobile()) {
    await reloadApp();
    return;
  }
  await restartAndroidApp(WEBDRIVER_PORT);
  // The old app process (and its WebDriver server) is gone; bind to the new one.
  await browser.reloadSession();
  await waitForAppReady();
};

/**
 * Resets the app to a pristine first-run state: closes and cold-restarts the app
 * (and the browser), then clears the Rust-persisted instance URL, auth token,
 * user info and avatar and wipes webview storage so the layout guard lands on
 * the setup page. Call between tests so every test starts from a clean,
 * freshly-launched app.
 */
export const resetAppData = async (): Promise<void> => {
  await restartApp();

  await invokeCommand('logout');
  await invokeCommand('reset_setup');

  await browser.deleteAllCookies().catch(() => {});
  await browser.execute(() => {
    try {
      localStorage.clear();
      sessionStorage.clear();
    } catch {
      // storage may be unavailable in some webviews; ignore
    }
  });

  await reloadApp();
  await waitForRoute('/setup');
};

/**
 * Seeds a completed setup with the given instance URL (bypassing the form's
 * server health check) and reloads. With setup done but no auth token, the
 * layout guard lands on `/auth`.
 */
export const seedSetup = async (url: string): Promise<void> => {
  await invokeCommand('setup', { url });
  await reloadApp();
  await waitForRoute('/auth');
};

/** Button whose visible text contains `text`. */
export const byButton = (text: string) => $(`button*=${text}`);

/** Any element whose text contains `text`. */
export const byText = (text: string) => $(`*=${text}`);

export const waitForElement = async (selector: string, timeout = 5000) => {
  const element = $(selector);
  await element.waitForExist({ timeout });
  return element;
};

export const waitForElementVisible = async (
  selector: string,
  timeout = 5000
) => {
  const element = $(selector);
  await element.waitForDisplayed({ timeout });
  return element;
};

export const waitForText = async (text: string, timeout = 5000) =>
  waitForElementVisible(`*=${text}`, timeout);

/**
 * Waits until the rendered page text contains `text`. Uses `innerText` instead
 * of a `*=` text selector because Svelte splits interpolated markup (e.g.
 * `URL: {url}`) into separate text nodes, which the WebDriver `contains(text())`
 * selector only matches on the first node.
 */
export const waitForBodyText = async (
  text: string,
  timeout = 5000
): Promise<void> => {
  await browser.waitUntil(
    async () =>
      (await browser.execute(() => document.body.textContent)).includes(text),
    { timeout, timeoutMsg: `Page text never contained "${text}"` }
  );
};

/**
 * Delivers a `positron://` deep link and waits for the app to settle on
 * `route`, re-firing the link if it doesn't. The deep-link handlers react by
 * pushing an updater message that the layout turns into a `goto`; if that push
 * races the updater channel reconnecting after a reload it can be dropped, so a
 * second delivery (the exchange is idempotent against the mock) makes the
 * navigation deterministic instead of flaky.
 */
export const openDeepLinkUntilRoute = async (
  url: string,
  route: string,
  attempts = 3,
  perAttempt = 6000
): Promise<void> => {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    openDeepLink(url);
    try {
      // eslint-disable-next-line no-await-in-loop -- sequential retry is intentional
      await waitForRoute(route, perAttempt);
      return;
    } catch {
      // Push likely lost to the reconnect race; deliver the link again.
    }
  }
  await waitForRoute(route, perAttempt);
};

/**
 * Fast path to an authenticated app for specs that only need the logged-in
 * state (not the browser UI round-trip): seeds a PKCE verifier via `start_auth`,
 * then delivers the `positron://auth` deep link directly through adb the way the
 * external browser would. Use {@link loginViaBrowser} when the goal is to
 * exercise the real `/auth` page button + browser hop itself.
 */
export const authenticateViaDeepLink = async (
  code = 'e2e-auth-code'
): Promise<void> => {
  await invokeCommand('start_auth');
  await openDeepLinkUntilRoute(`positron://auth?code=${code}`, '/');
};

/**
 * Drives the full real login from the `/auth` page: presses "Login", which runs
 * `start_auth` and opens the mock login page in the system browser via
 * `openUrl`. The page records the PKCE challenge and tries to bounce straight
 * back through the `positron://auth` deep link; the app exchanges the code and
 * lands authenticated on the main page — a real user sign-in, end to end.
 *
 * Chrome throttles gesture-free redirects to a custom scheme across repeated
 * visits, so the page's automatic JS hop can't be guaranteed on every run. We
 * always verify the real, deterministic part — the button opened the browser at
 * the instance URL carrying a fresh challenge — then give the JS redirect a
 * chance, and only deliver the return deep link ourselves if Chrome suppressed
 * it. Either way the app authenticates through the genuine exchange. Android
 * only (needs the browser + custom-scheme deep link).
 */
export const loginViaBrowser = async (): Promise<void> => {
  await byButton('Login').click();

  // The browser fetched the mock login page with the challenge the app minted.
  await waitForMockAuthStart();

  try {
    // Best case: the page's JS redirect fired and the app is already back.
    await waitForRouteThroughBackground('/', 8000);
  } catch {
    // Chrome suppressed the automatic external-scheme redirect; stand in for it
    // by delivering the same deep link the page would have, as the OS would.
    await openDeepLinkUntilRoute(`positron://auth?code=${MOCK_AUTH_CODE}`, '/');
  }
};

export const generateTestId = (prefix: string): string =>
  `${prefix}-${Date.now()}-${Math.random().toString(36).substring(7)}`;

export const takeScreenshotAsBase64 = async (): Promise<string> =>
  browser.takeScreenshot();

export const isValidBase64Png = (base64String: string): boolean => {
  try {
    const buffer = Buffer.from(base64String, 'base64');
    // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
    const pngMagic = Buffer.from([
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a
    ]);
    return buffer.subarray(0, 8).equals(pngMagic);
  } catch {
    return false;
  }
};

export const isValidBase64Pdf = (base64String: string): boolean => {
  try {
    const buffer = Buffer.from(base64String, 'base64');
    // PDF magic bytes: %PDF
    const pdfMagic = Buffer.from('%PDF');
    return buffer.subarray(0, 4).equals(pdfMagic);
  } catch {
    return false;
  }
};
