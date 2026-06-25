// oxlint-disable sort-keys no-console no-underscore-dangle
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  ensureAndroidAppRunning,
  startApp,
  stopApp
} from './helpers/app-launcher.js';
import { startMockServer, stopMockServer } from './helpers/mock-server.js';
import { mergeFiles } from 'junit-report-merger';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WEBDRIVER_PORT = 4445;

export const config: WebdriverIO.Config = {
  runner: 'local',

  tsConfigPath: './tsconfig.json',

  specs: [path.resolve(__dirname, 'specs', '*.spec.ts')],

  exclude: [],

  maxInstances: 1,

  capabilities: [
    {
      browserName: 'chrome',
      'goog:chromeOptions': {
        // We don't actually use Chrome - WebdriverIO connects to our custom WebDriver server
      }
    }
  ],

  // Connect to our WebDriver server
  hostname: '127.0.0.1',
  port: WEBDRIVER_PORT,
  path: '/',

  logLevel: 'warn',

  bail: 0,

  // A spec can fail wholesale in CI when the emulator crashes the app on a cold
  // restart; retrying the file (with the beforeSession relaunch below) recovers
  // from a one-off crash instead of failing the run.
  specFileRetries: 1,
  specFileRetriesDeferred: false,

  waitforTimeout: 10_000,

  connectionRetryTimeout: 120_000,

  connectionRetryCount: 3,

  framework: 'mocha',

  reporters: [
    'spec',
    [
      'junit',
      {
        outputDir: path.resolve(__dirname, 'reports'),
        outputFileFormat: (options) => `results-${options.cid}.xml`
      }
    ]
  ],

  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000
  },

  // Hooks
  onPrepare: async () => {
    console.log('Starting mock backend...');
    await startMockServer();

    console.log('Starting Tauri application...');
    await startApp(WEBDRIVER_PORT);
  },

  onComplete: async () => {
    console.log('Stopping Tauri application...');
    stopApp(WEBDRIVER_PORT);

    console.log('Stopping mock backend...');
    await stopMockServer();

    const sourceDir = path.resolve(__dirname, 'reports', '*.xml');
    const destFile = path.resolve(__dirname, 'reports', 'app-e2e-tests.xml');

    await mergeFiles(destFile, [sourceDir]);
  },

  beforeSession: async () => {
    // Each spec file (and each retry) gets its own session. If a previous spec
    // left the app dead — as a CI emulator crash can — relaunch it so this
    // worker can bind a session instead of failing on a refused port.
    if (process.env.TAURI_TEST_PLATFORM === 'android') {
      await ensureAndroidAppRunning(WEBDRIVER_PORT);
    }

    // Wait a bit for any lingering state to clear
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
};
