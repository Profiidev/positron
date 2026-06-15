import { describe, expect, it } from 'vitest';
import { load as layoutLoad } from '$routes/+layout';
import { load as layoutServerLoad } from '$routes/+layout.server';
import { catchRedirect, jsonFetch } from '../_helpers/load';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const ev = (props: Record<string, unknown>) => props as any;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const runLayout = async (e: unknown): Promise<any> => layoutLoad(e as never);

describe('+layout.ts load', () => {
  it('parses auth and oauth options from the query string', async () => {
    const result = await runLayout(
      ev({
        fetch: jsonFetch(null),
        url: new URL('http://x/?auth=login&challenge=c1&code=co&name=na')
      })
    );
    expect(result.auth).toEqual({ authType: 'login', challenge: 'c1' });
    expect(result.oauthOptions).toEqual({ code: 'co', name: 'na' });
  });

  it('derives the auth type from an /auth/* pathname when no auth param', async () => {
    const result = await runLayout(
      ev({ fetch: jsonFetch(null), url: new URL('http://x/auth/app') })
    );
    expect(result.auth.authType).toBe('app');
  });

  it('falls back to an Unknown User when info returns no data', async () => {
    const result = await runLayout(
      ev({ fetch: jsonFetch(null), url: new URL('http://x/') })
    );
    await expect(result.user).resolves.toMatchObject({
      name: 'Unknown User',
      permissions: []
    });
  });

  it('resolves the real user when info returns data', async () => {
    const user = {
      email: 'a@b.com',
      name: 'Bob',
      permissions: ['user:view'],
      totp_enabled: false,
      uuid: 'u1'
    };
    const result = await runLayout(
      ev({ fetch: jsonFetch(user), url: new URL('http://x/') })
    );
    await expect(result.user).resolves.toMatchObject({ name: 'Bob' });
  });
});

const cookies = (value?: string) => ({ get: () => value });

describe('+layout.server.ts load', () => {
  it('redirects to /login when unauthenticated on a protected path', async () => {
    const redirect = await catchRedirect(async () =>
      layoutServerLoad(
        ev({ cookies: cookies(), url: new URL('http://x/users') })
      )
    );
    expect(redirect).toMatchObject({ location: '/login', status: 302 });
  });

  it('allows an unauthenticated user on a public path', async () => {
    const result = await layoutServerLoad(
      ev({ cookies: cookies(), url: new URL('http://x/login') })
    );
    expect(result).toBeUndefined();
  });

  it('allows an authenticated user anywhere', async () => {
    const result = await layoutServerLoad(
      ev({ cookies: cookies('jwt'), url: new URL('http://x/users') })
    );
    expect(result).toBeUndefined();
  });
});
