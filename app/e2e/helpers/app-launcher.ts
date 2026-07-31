import { type ChildProcess, spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { existsSync } from 'node:fs';
import { MOCK_PORT } from './mock-server.js';

const currentFilename = fileURLToPath(import.meta.url);
const currentDirname = path.dirname(currentFilename);

// Unbundled binary name (Cargo package `positron`, see src-tauri/Cargo.toml).
const BINARY_NAME = 'positron';
// Bundled macOS app name comes from the Tauri `productName`.
const PRODUCT_NAME = 'Positron';

let appProcess: ChildProcess | undefined = undefined;
let adbPortForwarded = false;

type Platform = 'desktop' | 'android' | 'ios';

// Android app package name from tauri.conf.json identifier
const ANDROID_PACKAGE = 'io.profidev.positron';
const ANDROID_ACTIVITY = '.MainActivity';
// System browser the app's `openUrl` hands the login page to.
const ANDROID_BROWSER = 'com.android.chrome';

const getPlatform = (): Platform => {
  const env = process.env.TAURI_TEST_PLATFORM;
  if (env === 'android') {
    return 'android';
  }
  if (env === 'ios') {
    return 'ios';
  }
  return 'desktop';
};

const getAdbPath = (): string => {
  const androidHome = process.env.ANDROID_HOME;
  if (!androidHome) {
    throw new Error('ANDROID_HOME environment variable is not set');
  }

  const adbPath = path.join(
    androidHome,
    'platform-tools',
    process.platform === 'win32' ? 'adb.exe' : 'adb'
  );
  if (!existsSync(adbPath)) {
    throw new Error(`adb not found at ${adbPath}`);
  }

  return adbPath;
};

const runAdb = (args: string[]): { success: boolean; output: string } => {
  const adb = getAdbPath();
  console.log(`[adb] ${args.join(' ')}`);

  const result = spawnSync(adb, args, { encoding: 'utf8' });

  if (result.error) {
    console.error(`[adb error]: ${result.error.message}`);
    return { output: result.error.message, success: false };
  }

  const output = (result.stdout || '') + (result.stderr || '');
  if (result.status !== 0) {
    console.error(`[adb failed]: ${output}`);
    return { output, success: false };
  }

  return { output: output.trim(), success: true };
};

const setupAdbPortForward = (port: number): void => {
  const result = runAdb(['forward', `tcp:${port}`, `tcp:${port}`]);
  if (!result.success) {
    throw new Error(`Failed to set up adb port forwarding: ${result.output}`);
  }
  adbPortForwarded = true;
  console.log(`Port forwarding set up: localhost:${port} -> device:${port}`);
};

const removeAdbPortForward = (port: number): void => {
  if (!adbPortForwarded) {
    return;
  }

  runAdb(['forward', '--remove', `tcp:${port}`]);
  adbPortForwarded = false;
  console.log(`Port forwarding removed for port ${port}`);
};

// `reverse` makes the device's localhost:<port> tunnel back to the host's
// localhost:<port>, so the mock backend (running on the host) is reachable from
// the app inside the emulator using the same `127.0.0.1` URL as on desktop.
const setupAdbReverse = (port: number): void => {
  const result = runAdb(['reverse', `tcp:${port}`, `tcp:${port}`]);
  if (!result.success) {
    throw new Error(`Failed to set up adb reverse: ${result.output}`);
  }
  console.log(`Reverse tunnel set up: device:${port} -> localhost:${port}`);
};

const removeAdbReverse = (port: number): void => {
  runAdb(['reverse', '--remove', `tcp:${port}`]);
  console.log(`Reverse tunnel removed for port ${port}`);
};

const startAndroidApp = (): void => {
  const component = `${ANDROID_PACKAGE}/${ANDROID_ACTIVITY}`;
  const result = runAdb(['shell', 'am', 'start', '-n', component]);
  if (!result.success) {
    throw new Error(`Failed to start Android app: ${result.output}`);
  }
  console.log(`Started Android app: ${component}`);
};

const stopAndroidApp = (): void => {
  runAdb(['shell', 'am', 'force-stop', ANDROID_PACKAGE]);
  console.log(`Stopped Android app: ${ANDROID_PACKAGE}`);
};

const stopAndroidBrowser = (): void => {
  runAdb(['shell', 'am', 'force-stop', ANDROID_BROWSER]);
  console.log(`Stopped Android browser: ${ANDROID_BROWSER}`);
};

const resetAppData = (): void => {
  runAdb(['shell', 'pm', 'clear', ANDROID_PACKAGE]);
  console.log('App data reset');
};

/**
 * Fully closes the app and the system browser, then relaunches the app cold and
 * waits for its WebDriver server to come back up. Used to isolate every test in
 * a fresh process (no backgrounded state, no leftover browser tab) instead of
 * reusing the running instance. The caller must follow this with
 * `browser.reloadSession()` to bind WebDriver to the new app process.
 */
export const restartAndroidApp = async (port = 4445): Promise<void> => {
  stopAndroidApp();
  stopAndroidBrowser();
  resetAppData();
  startAndroidApp();
  await waitForServer(port);
};

/**
 * Ensures the Android app (and its embedded WebDriver server) is up before a
 * worker creates its session. The app is started once in `onPrepare`, but in CI
 * it can crash during a cold restart and never come back; without this every
 * following spec fails to create a session on the now-dead port, cascading one
 * flaky crash into a whole-suite failure. `am start` is idempotent (it just
 * foregrounds a live app), so calling this per session is safe and lets each
 * spec file — and each retry — self-heal instead of inheriting a dead process.
 */
export const ensureAndroidAppRunning = async (port = 4445): Promise<void> => {
  startAndroidApp();
  await waitForServer(port);
};

/**
 * Fires an OS deep link at the app, the way the real auth flow delivers
 * `positron://auth?...` / `positron://login?...` URLs back to the app after the
 * external browser step.
 *
 * - Android: sent via `adb`. The URL is single-quoted so the device shell
 *   keeps `&` query separators intact instead of backgrounding the command.
 * - Desktop: the OS equivalent of a registered scheme handler invoking the app
 *   again is a second process launched with the URL as argv. `setup_deep_link`
 *   registers the scheme at runtime on Linux/Windows debug builds, and
 *   `tauri-plugin-single-instance`'s `deep-link` feature (see
 *   `src-tauri/Cargo.toml`) forwards that argv to the already-running instance
 *   before this short-lived process exits, so no adb-style bridge is needed.
 */
export const openDeepLink = (url: string): void => {
  const platform = getPlatform();

  if (platform === 'android') {
    const result = runAdb([
      'shell',
      'am',
      'start',
      '-W',
      '-a',
      'android.intent.action.VIEW',
      '-d',
      `'${url}'`,
      ANDROID_PACKAGE
    ]);
    if (!result.success) {
      throw new Error(`Failed to open deep link ${url}: ${result.output}`);
    }
    console.log(`Opened deep link: ${url}`);
    return;
  }

  if (platform === 'desktop') {
    const appPath = getAppPath();
    const child = spawn(appPath, [url], {
      env: desktopAppEnv(),
      stdio: 'ignore'
    });
    child.unref();
    console.log(`Opened deep link via relaunch: ${url}`);
    return;
  }

  throw new Error(
    `openDeepLink is not implemented for platform ${platform}`
  );
};

// Forces the X11 GDK backend. Without this, a spawned GTK app inherits
// whatever `WAYLAND_DISPLAY`/session type the parent shell has (a Wayland
// desktop locally, or a leaked env var in CI) and tries to connect there
// instead of the X server (real or Xvfb) the app-launcher actually targets,
// crashing immediately with "Gdk-Message: Error 71 (Protocol error)
// dispatching to Wayland display."
const desktopAppEnv = (): NodeJS.ProcessEnv => ({
  ...process.env,
  GDK_BACKEND: 'x11'
});

const resolveBinaryInBase = (base: string): string => {
  switch (process.platform) {
    case 'darwin': {
      // Try bundled app first, fall back to unbundled binary (--no-bundle)
      const bundledPath = path.resolve(
        base,
        `bundle/macos/${PRODUCT_NAME}.app/Contents/MacOS/${BINARY_NAME}`
      );
      const unbundledPath = path.resolve(base, BINARY_NAME);
      return existsSync(bundledPath) ? bundledPath : unbundledPath;
    }
    case 'win32': {
      return path.resolve(base, `${BINARY_NAME}.exe`);
    }
    case 'linux': {
      return path.resolve(base, BINARY_NAME);
    }
    default: {
      throw new Error(`Unsupported platform: ${process.platform}`);
    }
  }
};

// `app/src-tauri` is a member of the root Cargo workspace (see /Cargo.toml), so
// cargo (and thus `tauri build`) puts binaries in the *workspace* target dir at
// the repo root, not `app/src-tauri/target`.
const WORKSPACE_TARGET_DIR = path.resolve(currentDirname, '../../../target');

/**
 * Resolves the built app binary, preferring a `--debug` build (what
 * `npm run build:linux` produces) and falling back to a `--release` one.
 */
export const getAppPath = (): string => {
  const debugPath = resolveBinaryInBase(
    path.resolve(WORKSPACE_TARGET_DIR, 'debug')
  );
  if (existsSync(debugPath)) {
    return debugPath;
  }

  return resolveBinaryInBase(path.resolve(WORKSPACE_TARGET_DIR, 'release'));
};

const waitForServer = async (port: number, timeout = 30_000): Promise<void> => {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    try {
      // eslint-disable-next-line no-await-in-loop -- sequential polling is intentional
      const response = await fetch(`http://127.0.0.1:${port}/status`);
      if (response.ok) {
        console.log(`WebDriver server ready on port ${port}`);
        return;
      }
    } catch {
      // Server not ready yet
    }
    // eslint-disable-next-line no-await-in-loop -- sequential polling is intentional
    await new Promise((done) => setTimeout(done, 500));
  }

  throw new Error(`WebDriver server did not start within ${timeout}ms`);
};

export const startApp = async (
  port = 4445
): Promise<ChildProcess | undefined> => {
  const platform = getPlatform();

  if (platform === 'android') {
    console.log('Setting up Android test environment...');

    // Set up port forwarding from host to device
    setupAdbPortForward(port);

    // Tunnel the host mock backend into the emulator
    setupAdbReverse(MOCK_PORT);

    // Start the Android app
    startAndroidApp();

    // Wait for WebDriver server to be ready
    await waitForServer(port);
    return undefined;
  }

  if (platform === 'ios') {
    // IOS - just wait for server, user handles app lifecycle
    console.log(`Waiting for iOS app on port ${port}...`);
    await waitForServer(port);
    return undefined;
  }

  // Desktop - spawn app
  const appPath = getAppPath();
  console.log(`Starting Tauri app: ${appPath}`);

  appProcess = spawn(appPath, [], {
    env: {
      ...desktopAppEnv(),
      TAURI_WEBDRIVER_PORT: port.toString()
    },
    stdio: ['ignore', 'pipe', 'pipe']
  });

  appProcess.stdout?.on('data', (data) => {
    console.log(`[app stdout]: ${data.toString().trim()}`);
  });

  appProcess.stderr?.on('data', (data) => {
    console.error(`[app stderr]: ${data.toString().trim()}`);
  });

  appProcess.on('error', (err) => {
    console.error('Failed to start app:', err);
  });

  appProcess.on('exit', (code, signal) => {
    console.log(`App exited with code ${code}, signal ${signal}`);
    appProcess = undefined;
  });

  await waitForServer(port);

  return appProcess;
};

export const stopApp = (port = 4445): void => {
  const platform = getPlatform();

  if (platform === 'android') {
    console.log('Cleaning up Android test environment...');
    stopAndroidApp();
    stopAndroidBrowser();
    removeAdbReverse(MOCK_PORT);
    removeAdbPortForward(port);
    return;
  }

  if (platform === 'ios') {
    // IOS - nothing to do, user handles app lifecycle
    return;
  }

  // Desktop
  if (appProcess) {
    console.log('Stopping Tauri app...');
    appProcess.kill('SIGTERM');
    appProcess = undefined;
  }
};

export const getAppProcess = (): ChildProcess | undefined => appProcess;
