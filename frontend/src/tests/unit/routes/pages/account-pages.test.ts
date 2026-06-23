import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import General from '$routes/account/general/+page.svelte';
import Settings from '$routes/account/settings/+page.svelte';
import Auth from '$routes/account/auth/+page.svelte';
import Sessions from '$routes/account/sessions/+page.svelte';

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

  it('sessions renders its heading and current device badge', async () => {
    r(Sessions, {
      sessions: pr([
        {
          application: 'Chrome 126',
          created_at: new Date('2024-01-01T00:00:00Z'),
          current: true,
          expires_at: new Date('2024-07-01T00:00:00Z'),
          id: 'session-1',
          is_app: false,
          last_used_at: new Date('2024-06-01T00:00:00Z'),
          name: 'MacBook Pro',
          operating_system: 'macOS 15.1',
          refreshed_at: new Date('2024-06-01T00:00:00Z')
        }
      ])
    });
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(await screen.findByText('This device')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Revoke all other sessions' })
    ).toBeDisabled();
  });

  it('enables bulk revoke when a non-current session exists', async () => {
    const base = {
      application: 'Chrome 126',
      created_at: new Date('2024-01-01T00:00:00Z'),
      expires_at: new Date('2024-07-01T00:00:00Z'),
      is_app: false,
      last_used_at: new Date('2024-06-01T00:00:00Z'),
      name: 'MacBook Pro',
      operating_system: 'macOS 15.1',
      refreshed_at: new Date('2024-06-01T00:00:00Z')
    };
    r(Sessions, {
      sessions: pr([
        { ...base, current: true, id: 'session-1' },
        { ...base, current: false, id: 'session-2', name: 'iPhone 15 Pro' }
      ])
    });
    expect(
      await screen.findByRole('button', {
        name: 'Revoke all other sessions'
      })
    ).toBeEnabled();
  });
});
