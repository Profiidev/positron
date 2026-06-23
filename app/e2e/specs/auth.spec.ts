import { expect } from '@wdio/globals';
import { MOCK_AUTH_CODE, MOCK_URL } from '../helpers/mock-server.js';
import {
  byButton,
  getMockState,
  getRoute,
  loginViaBrowser,
  resetAppData,
  resetMockState,
  seedSetup,
  waitForBodyText,
  waitForRoute
} from '../helpers/test-utils.js';

describe('Auth page', () => {
  beforeEach(async () => {
    await resetMockState();
    await resetAppData();
    await seedSetup(MOCK_URL);
  });

  it('shows the configured instance URL', async () => {
    await waitForBodyText(MOCK_URL);
  });

  it('offers a Login button', async () => {
    await expect(byButton('Login')).toBeDisplayed();
  });

  it('returns to setup when changing the instance', async () => {
    await byButton('Change').click();
    await waitForRoute('/setup');
    expect(await getRoute()).toBe('/setup');
  });

  describe('full browser login flow', () => {
    // The login button opens the mock login page in the system browser, which
    // bounces back via a `positron://auth` deep link — only wired up on Android.
    // oxlint-disable-next-line func-names
    before(function () {
      if (process.env.TAURI_TEST_PLATFORM !== 'android') {
        this.skip();
      }
    });

    // One real round-trip (press Login -> browser login page -> deep link back),
    // asserting every link in the chain: the app authenticates, the page saw the
    // real PKCE challenge, and the native client exchanged the returned code.
    it('authenticates end to end through the browser', async () => {
      await loginViaBrowser();

      expect(await getRoute()).toBe('/');
      await expect(byButton('Logout')).toBeDisplayed();

      const mock = await getMockState();
      // base64url-no-pad of a SHA-256 digest is exactly 43 chars.
      expect(mock.authChallenge).toMatch(/^[A-Za-z0-9_-]{43}$/);
      expect(mock.exchangeCount).toBeGreaterThan(0);
      expect(mock.exchange?.code).toBe(MOCK_AUTH_CODE);
      // The native client must echo the PKCE verifier it stored on start_auth.
      expect(mock.exchange?.verifier).toBeDefined();
    });
  });
});
