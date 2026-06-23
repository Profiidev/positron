import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const logout = vi.fn();
vi.mock('$lib/commands/auth.svelte', () => ({ logout }));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte', () => ({ isConnected }));

const Page = (await import('$routes/+page.svelte')).default;

describe('home page', () => {
  it('logs out when the Logout button is clicked', async () => {
    render(Page);
    screen.getByRole('button', { name: 'Logout' }).click();
    await vi.waitFor(() => expect(logout).toHaveBeenCalled());
  });

  it('navigates to /scan when the Scan button is clicked', async () => {
    render(Page);
    screen.getByRole('button', { name: 'Scan Login Code' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/scan'));
  });

  it('hides the Disconnected badge while connected', () => {
    isConnected.mockReturnValue(true);
    render(Page);
    expect(screen.queryByText('Disconnected')).not.toBeInTheDocument();
  });

  it('shows the Disconnected badge while disconnected', () => {
    isConnected.mockReturnValue(false);
    render(Page);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });
});
