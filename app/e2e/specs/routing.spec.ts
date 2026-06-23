import { expect } from '@wdio/globals';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  byButton,
  getRoute,
  resetAppData,
  seedSetup
} from '../helpers/test-utils.js';

describe('Routing guards', () => {
  beforeEach(async () => {
    await resetAppData();
  });

  it('redirects an unconfigured app to setup', async () => {
    expect(await getRoute()).toBe('/setup');
  });

  it('redirects a configured but unauthenticated app to auth', async () => {
    await seedSetup(MOCK_URL);
    expect(await getRoute()).toBe('/auth');
  });

  it('keeps unauthenticated users out of the main page', async () => {
    await seedSetup(MOCK_URL);
    // With no auth token the main page (its Logout button) must never render.
    await expect(byButton('Logout')).not.toBeDisplayed();
    expect(await getRoute()).toBe('/auth');
  });
});
