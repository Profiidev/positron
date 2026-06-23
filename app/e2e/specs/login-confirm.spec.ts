import { browser, expect } from '@wdio/globals';
import { openDeepLink } from '../helpers/app-launcher.js';
import { MOCK_URL, MOCK_USER } from '../helpers/mock-server.js';
import {
  authenticateViaDeepLink,
  byButton,
  getMockState,
  getRoute,
  openDeepLinkUntilRoute,
  resetAppData,
  resetMockState,
  seedSetup,
  waitForBodyText,
  waitForRoute
} from '../helpers/test-utils.js';

/**
 * The `/login` confirmation page is what a scanned QR login code lands on: a
 * `positron://login?code=...` deep link (the same payload the scanner extracts)
 * routes the authenticated app here to approve a second device's sign-in. The
 * camera scan itself can't run headless (see scan note in authenticated.spec),
 * so the deep link stands in for it.
 */
describe('Login confirmation page', () => {
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

  // oxlint-disable-next-line consistent-function-scoping
  const openLogin = async (query = 'code=e2e-login-code'): Promise<void> => {
    await openDeepLinkUntilRoute(`positron://login?${query}`, '/login');
  };

  it('shows the logged-in account to confirm', async () => {
    await openLogin();

    await waitForBodyText(MOCK_USER.name);
    await waitForBodyText(MOCK_USER.email);
  });

  it('approves the login code and returns to the main page', async () => {
    await openLogin();
    await byButton('Confirm').click();
    await waitForRoute('/');

    const mock = await getMockState();
    expect(mock.approveCount).toBeGreaterThan(0);
    expect(mock.approve?.code).toBe('e2e-login-code');
  });

  it('cancels without approving and returns to the main page', async () => {
    await openLogin();
    await byButton('Cancel').click();
    await waitForRoute('/');

    const mock = await getMockState();
    expect(mock.approveCount).toBe(0);
  });

  it('confirms a code that carries a same-origin redirect', async () => {
    await openLogin(
      `code=redirect-code&redirect=${encodeURIComponent(`${MOCK_URL}/done`)}`
    );
    await byButton('Confirm').click();
    await waitForRoute('/');

    const mock = await getMockState();
    expect(mock.approve?.code).toBe('redirect-code');
  });

  it('changes account from the confirmation page back to auth', async () => {
    await openLogin();
    await byButton('Change').click();
    await waitForRoute('/auth');
  });

  it('ignores a login deep link that carries no code', async () => {
    // Missing code aborts confirmation: the app stays on the main page and never
    // routes to /login.
    openDeepLink('positron://login');
    await browser.pause(2000);

    expect(await getRoute()).toBe('/');
    const mock = await getMockState();
    expect(mock.approveCount).toBe(0);
  });
});
