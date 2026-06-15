import { describe, expect, it } from 'vitest';
import { load as authLoad } from '$routes/auth/+page.server';
import { load as accountLoad } from '$routes/account/+page.server';
import { load as passwordLoad } from '$routes/password/+page.server';
import { load as settingsLoad } from '$routes/settings/+page.server';
import { load as oauthLogoutLoad } from '$routes/oauth/logout/+page.server';
import { catchRedirect, runLoad } from '$test_helpers/load';

describe('index redirects', () => {
  it.each([
    ['auth', authLoad, '/'],
    ['account', accountLoad, '/account/general'],
    ['password', passwordLoad, '/password/forgot'],
    ['settings', settingsLoad, '/settings/mail']
  ])('%s redirects to %s', async (_name, load, location) => {
    const redirect = await catchRedirect(() => (load as () => unknown)());
    expect(redirect.status).toBe(302);
    expect(redirect.location).toBe(location);
  });
});

describe('oauth logout load', () => {
  it('returns the logout target when url and name are present', async () => {
    const result = await runLoad(oauthLogoutLoad, {
      url: new URL('http://x/oauth/logout?url=https://app/cb&name=App')
    });
    expect(result.oauthLogout).toEqual({ name: 'App', url: 'https://app/cb' });
  });

  it('returns undefined when either param is missing', async () => {
    const result = await runLoad(oauthLogoutLoad, {
      url: new URL('http://x/oauth/logout?url=https://app/cb')
    });
    expect(result.oauthLogout).toBeUndefined();
  });
});
