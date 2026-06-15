import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Forgot from '$routes/password/forgot/+page.svelte';
import Reset from '$routes/password/reset/+page.svelte';
import Oauth from '$routes/oauth/+page.svelte';
import AuthApp from '$routes/auth/app/+page.svelte';
import Logout from '$routes/oauth/logout/+page.svelte';

const P =  async <T,>(v: T) => Promise.resolve(v);
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const r = (Cmp: any, data: unknown) => render(Cmp, { data } as any);

describe('auth-flow pages', () => {
  it('forgot password renders its card', () => {
    r(Forgot, {});
    expect(screen.getByText('Forgot Password')).toBeInTheDocument();
  });

  it('reset password renders its card', () => {
    r(Reset, { token: 'tok' });
    expect(screen.getAllByText('Reset Password').length).toBeGreaterThan(0);
  });

  it('oauth consent renders the login prompt', () => {
    r(Oauth, {
      location: null,
      oauthOptions: { code: 'c', name: 'App' },
      settings: P({ o_auth_instant_confirm: false }),
      user: P(undefined)
    });
    expect(screen.getByText(/Log in to/)).toBeInTheDocument();
  });

  it('auth app renders the app login prompt', () => {
    r(AuthApp, {
      auth: { authType: 'app', challenge: 'c' },
      code: null,
      user: P(undefined)
    });
    expect(screen.getByText(/Log in to Positron App/)).toBeInTheDocument();
  });

  it('oauth logout renders without a target', () => {
    const { container } = r(Logout, {
      oauthLogout: undefined,
      user: P(undefined)
    });
    expect(container.innerHTML.length).toBeGreaterThan(0);
  });
});
