import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import General from '$routes/account/general/+page.svelte';
import Settings from '$routes/account/settings/+page.svelte';
import Auth from '$routes/account/auth/+page.svelte';

const pr = async <T>(v: T) => Promise.resolve(v);
const user = {
  email: 'a@b.com',
  name: 'Bob',
  permissions: [],
  totp_enabled: false,
  uuid: 'u1'
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const r = (Cmp: any, data: unknown) => render(Cmp, { data } as any);

describe('account pages', () => {
  it('general settings renders its heading and username field', () => {
    r(General, { user: pr(user) });
    expect(screen.getByText('General Settings')).toBeInTheDocument();
    expect(screen.getByText('Username')).toBeInTheDocument();
  });

  it('settings renders its heading', () => {
    r(Settings, { settings: pr({ o_auth_instant_confirm: false }) });
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('auth renders its heading', () => {
    r(Auth, { mailActive: pr(false), passkeys: pr([]), user: pr(user) });
    expect(screen.getByText('Authentication')).toBeInTheDocument();
  });
});
