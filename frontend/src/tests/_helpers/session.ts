import type { BrowserContext } from '@playwright/test';
import type { Scenario } from '../mocks/e2e/data';

const URL = 'http://localhost:4173';

/**
 * Seeds the auth cookie (so protected routes don't redirect to /login) and the
 * `mock_scenario` cookie that the e2e MSW handlers read to vary their data.
 */
export const setupSession = async (
  context: BrowserContext,
  scenario: Scenario = 'default'
) =>
  context.addCookies([
    { name: 'centaurus_jwt', url: URL, value: 'e2e-token' },
    { name: 'mock_scenario', url: URL, value: scenario }
  ]);
