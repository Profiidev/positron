import type { BrowserContext } from '@playwright/test';
import type { Scenario } from '../mocks/e2e/data';

const URL = 'http://localhost:4173';

/**
 * Mirrors a cookie into `document.cookie` for every page in the context. The
 * e2e MSW handlers read `mock_scenario` / `mock_setup` from the *client-side*
 * request cookies; WebKit does not reliably expose context cookies to those
 * intercepted fetches, so we also seed them through an init script to make the
 * scenario deterministic across every browser project.
 */
const seedDocumentCookie = async (context: BrowserContext, cookie: string) =>
  context.addInitScript((value) => {
    // oxlint-disable-next-line no-document-cookie
    document.cookie = `${value}; path=/`;
  }, cookie);

/**
 * Seeds the auth cookie (so protected routes don't redirect to /login) and the
 * `mock_scenario` cookie that the e2e MSW handlers read to vary their data.
 */
export const setupSession = async (
  context: BrowserContext,
  scenario: Scenario = 'default'
) => {
  await context.addCookies([
    { name: 'centaurus_jwt', url: URL, value: 'e2e-token' },
    { name: 'mock_scenario', url: URL, value: scenario }
  ]);
  await seedDocumentCookie(context, `mock_scenario=${scenario}`);
};

/**
 * Seeds only the `mock_setup=pending` cookie (and no auth cookie) so the
 * `isSetup` endpoint reports an un-provisioned instance. Used by the /setup
 * tests, where the first-run wizard must render instead of redirecting away.
 */
export const seedSetupPending = async (context: BrowserContext) => {
  await context.addCookies([
    { name: 'mock_setup', url: URL, value: 'pending' }
  ]);
  await seedDocumentCookie(context, 'mock_setup=pending');
};
