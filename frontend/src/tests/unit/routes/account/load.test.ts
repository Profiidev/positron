import { describe, expect, it } from 'vitest';
import { load as authLoad } from '$routes/account/auth/+page';
import { load as sessionsLoad } from '$routes/account/sessions/+page';
import { load as settingsLoad } from '$routes/account/settings/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

describe('account auth load', () => {
  it('resolves passkeys and derives mailActive', async () => {
    const result = await runLoad(authLoad, {
      fetch: jsonFetch({ active: true })
    });
    await expect(result.mailActive).resolves.toBe(true);
    await expect(result.passkeys).resolves.toBeDefined();
  });

  it('defaults mailActive to false without data', async () => {
    const result = await runLoad(authLoad, { fetch: jsonFetch(null) });
    await expect(result.mailActive).resolves.toBe(false);
  });
});

describe('account settings load', () => {
  it('resolves the account settings', async () => {
    const result = await runLoad(settingsLoad, {
      fetch: jsonFetch({ o_auth_instant_confirm: true })
    });
    await expect(result.settings).resolves.toEqual({
      o_auth_instant_confirm: true
    });
  });
});

describe('account sessions load', () => {
  it('resolves the session list', async () => {
    const sessions = [
      {
        application: 'Chrome 126',
        created_at: '2024-01-01T00:00:00Z',
        current: true,
        expires_at: '2024-07-01T00:00:00Z',
        id: 'session-1',
        is_app: false,
        last_used_at: '2024-06-01T00:00:00Z',
        name: 'MacBook Pro',
        operating_system: 'macOS 15.1',
        refreshed_at: '2024-06-01T00:00:00Z'
      }
    ];
    const result = await runLoad(sessionsLoad, {
      fetch: jsonFetch(sessions)
    });
    await expect(result.sessions).resolves.toEqual(sessions);
  });
});
