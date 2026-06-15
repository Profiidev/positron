import { describe, expect, it } from 'vitest';
import { load as authLoad } from '$routes/account/auth/+page';
import { load as settingsLoad } from '$routes/account/settings/+page';
import { jsonFetch, runLoad } from '../../_helpers/load';

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
