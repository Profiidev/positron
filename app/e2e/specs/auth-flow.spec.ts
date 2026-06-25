import { browser, expect } from '@wdio/globals';
import { openDeepLink } from '../helpers/app-launcher.js';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  authenticateViaDeepLink,
  byButton,
  getMockState,
  getRoute,
  invokeCommand,
  resetAppData,
  resetMockState,
  seedSetup,
  waitForRoute
} from '../helpers/test-utils.js';

/**
 * Exercises the `positron://auth` deep-link half of authentication directly
 * (the browser hop in front of it is covered in auth.spec.ts). Delivering the
 * link via adb lets us drive the success path and the failure branches the UI
 * can't reach on its own.
 */
describe('Authentication deep link', () => {
  // Deep links are delivered through adb, so these only run on Android.
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
  });

  it('authenticates and lands on the main page', async () => {
    await authenticateViaDeepLink();

    expect(await getRoute()).toBe('/');
    await expect(byButton('Logout')).toBeDisplayed();
    await expect(byButton('Scan Login')).toBeDisplayed();
  });

  it('exchanges the deep-link code together with the stored verifier', async () => {
    await authenticateViaDeepLink('specific-auth-code');

    const mock = await getMockState();
    expect(mock.exchange?.code).toBe('specific-auth-code');
    expect(mock.exchange?.verifier).toBeDefined();
  });

  it('stays unauthenticated when the deep link carries no code', async () => {
    // Seed a verifier, then deliver a malformed link. The exchange must not run,
    // so the app never leaves /auth and the main page stays unreachable.
    await invokeCommand('start_auth');
    openDeepLink('positron://auth');

    await browser.pause(2000);
    expect(await getRoute()).toBe('/auth');
    await expect(byButton('Logout')).not.toBeDisplayed();

    const mock = await getMockState();
    expect(mock.exchangeCount).toBe(0);
  });

  it('logs out from the main page back to auth', async () => {
    await authenticateViaDeepLink();

    await byButton('Logout').click();
    await waitForRoute('/auth');
  });
});
