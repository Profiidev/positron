import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const logout = vi.fn(async () => true);
vi.mock('$lib/commands/auth.svelte', () => ({ logout }));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte', () => ({ isConnected }));

// Desktop builds set the __desktop__ Vite define at build time; a plain
// Static import can't flip that per-test the way `mockCommand` flips a Tauri
// Command, so this module (unlike nav.test.ts's plain mobile-default import)
// Gets its own file overriding `$lib/env` to isolate the desktop branch.
vi.mock('$lib/env', () => ({ IS_DESKTOP: true, IS_MOBILE: false }));

const Nav = (await import('$lib/components/Nav.svelte')).default;

afterEach(() => vi.clearAllMocks());

describe('Nav on desktop', () => {
  it('shows a Settings button instead of Scan Login', () => {
    render(Nav);
    expect(
      screen.getByRole('button', { name: 'Settings' })
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: 'Scan Login' })
    ).not.toBeInTheDocument();
  });

  it('navigates to /settings from the Settings button', async () => {
    render(Nav);
    screen.getByRole('button', { name: 'Settings' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/settings'));
  });
});
