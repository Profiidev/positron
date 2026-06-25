import { browser, expect } from '@wdio/globals';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  authenticateViaDeepLink,
  byButton,
  getRoute,
  resetAppData,
  resetMockState,
  seedSetup,
  waitForBodyText,
  waitForRoute
} from '../helpers/test-utils.js';

describe('Main page (authenticated)', () => {
  // oxlint-disable-next-line func-names
  before(function () {
    if (process.env.TAURI_TEST_PLATFORM !== 'android') {
      this.skip();
    }
  });

  beforeEach(async () => {
    await resetMockState();
    await resetAppData();
    await seedSetup(MOCK_URL);
    await authenticateViaDeepLink();
  });

  it('lands on the notes page with the nav', async () => {
    await browser.pause(500);
    expect(await getRoute()).toBe('/');
    // The nav exposes the global actions; the page itself shows the notes list.
    await expect(byButton('Logout')).toBeDisplayed();
    await expect(byButton('Scan Login')).toBeDisplayed();
    await waitForBodyText('Notes');
    await waitForBodyText('No notes yet');
  });

  it('shows a disconnected badge while the updater websocket is down', async () => {
    // The mock backend serves no `/api/ws/updater`, so the app's updater socket
    // fails to connect and the nav must surface the offline state.
    await waitForBodyText('Disconnected', 15_000);
  });

  it('logs out from the nav back to auth', async () => {
    await byButton('Logout').click();
    await waitForRoute('/auth');
  });

  it('keeps the user authenticated across a reload', async () => {
    await browser.execute(() => window.location.reload());
    await browser.pause(500);

    // A persisted, still-valid token must not bounce the user back to /auth.
    await expect(byButton('Logout')).toBeDisplayed();
    expect(await getRoute()).toBe('/');
  });
});

/*
 * The QR scan page (`/scan`) cannot be driven in an automated run: it needs a
 * real camera plus barcode-scanner permission, which the headless emulator
 * (`-camera-back none`) does not provide. Pressing "Scan Login Code" triggers a
 * native camera-permission prompt that the WebDriver session cannot dismiss, so
 * the button is intentionally never clicked above. The downstream behaviour it
 * produces — a `positron://login` deep link landing on `/login` — is covered in
 * login-confirm.spec.ts, and the scan component logic is covered by the Vitest
 * unit tests in app/src/tests/unit/routes/scan.
 *
 * Documented here as skipped so the coverage gap stays explicit.
 */
/*
describe.skip('Scan page (requires a real camera)', () => {
  it('scans a QR login code and routes to the login page');
});
*/
