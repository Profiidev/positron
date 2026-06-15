import { describe, expect, it } from 'vitest';
import { load } from '$routes/login/+page.server';
import { catchRedirect, runLoad } from '../../_helpers/load';

const cookies = (value?: string) => ({ get: () => value });

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const ev = (props: Record<string, unknown>) => props as any;

describe('login +page.server load', () => {
  it('does nothing when unauthenticated', async () => {
    const result = await runLoad(load, {
      cookies: cookies(),
      url: new URL('http://x/login')
    });
    expect(result).toBeUndefined();
  });

  it('redirects an authenticated user with oauth code+name to /oauth', async () => {
    const redirect = await catchRedirect(async () =>
      load(
        ev({
          cookies: cookies('jwt'),
          url: new URL('http://x/login?code=abc&name=App')
        })
      )
    );
    expect(redirect.location).toBe('/oauth?code=abc&name=App');
  });

  it('redirects an authenticated user with an auth param to /auth/*', async () => {
    const redirect = await catchRedirect(async () =>
      load(
        ev({ cookies: cookies('jwt'), url: new URL('http://x/login?auth=app') })
      )
    );
    expect(redirect.location).toBe('/auth/app');
  });

  it('redirects an authenticated user on /login to /', async () => {
    const redirect = await catchRedirect(async () =>
      load(ev({ cookies: cookies('jwt'), url: new URL('http://x/login') }))
    );
    expect(redirect.location).toBe('/');
  });

  it('does not redirect an authenticated user already off /login', async () => {
    const result = await runLoad(load, {
      cookies: cookies('jwt'),
      url: new URL('http://x/elsewhere')
    });
    expect(result).toBeUndefined();
  });
});
