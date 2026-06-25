import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const logout = vi.fn(async () => true);
vi.mock('$lib/commands/auth.svelte', () => ({ logout }));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte', () => ({ isConnected }));

const Nav = (await import('$lib/components/Nav.svelte')).default;

afterEach(() => vi.clearAllMocks());

describe('Nav', () => {
  it('navigates home from the Notes button', async () => {
    render(Nav);
    screen.getByRole('button', { name: 'Notes' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/'));
  });

  it('navigates to /scan from the Scan Login button', async () => {
    render(Nav);
    screen.getByRole('button', { name: 'Scan Login' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/scan'));
  });

  it('logs out from the Logout button', async () => {
    render(Nav);
    screen.getByRole('button', { name: 'Logout' }).click();
    await vi.waitFor(() => expect(logout).toHaveBeenCalled());
  });

  it('hides the Disconnected badge while connected', () => {
    isConnected.mockReturnValue(true);
    render(Nav);
    expect(screen.queryByText('Disconnected')).not.toBeInTheDocument();
  });

  it('shows the Disconnected badge while disconnected', () => {
    isConnected.mockReturnValue(false);
    render(Nav);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });
});
