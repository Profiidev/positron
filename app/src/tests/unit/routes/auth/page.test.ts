import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const startAuth = vi.fn();
const resetSetup = vi.fn();
vi.mock('$lib/commands/auth.svelte.js', () => ({ startAuth }));
vi.mock('$lib/commands/setup.svelte.js', () => ({ resetSetup }));

const openUrl = vi.fn(async (_url: URL | string) => {});
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl }));

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

const setupStatusState = { value: undefined as unknown };
vi.mock('$lib/updater/state.svelte', () => ({ setupStatusState }));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte', () => ({ isConnected }));

const Page = (await import('$routes/auth/+page.svelte')).default;

beforeEach(() => {
  setupStatusState.value = { url: 'https://example.com' };
  isConnected.mockReturnValue(true);
});
afterEach(() => vi.clearAllMocks());

describe('auth page', () => {
  it('shows the configured instance url', () => {
    render(Page);
    expect(screen.getByText(/https:\/\/example\.com/)).toBeInTheDocument();
  });

  it('opens the auth url with the challenge on success', async () => {
    startAuth.mockResolvedValue('challenge-123');
    render(Page);
    screen.getByRole('button', { name: 'Login' }).click();
    await vi.waitFor(() => expect(openUrl).toHaveBeenCalled());
    const url = openUrl.mock.calls[0][0] as URL;
    expect(url.pathname).toBe('/auth/app');
    expect(url.searchParams.get('challenge')).toBe('challenge-123');
  });

  it('toasts an error when the challenge cannot be obtained', async () => {
    startAuth.mockResolvedValue(undefined);
    render(Page);
    screen.getByRole('button', { name: 'Login' }).click();
    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('Failed to get auth challenge')
    );
    expect(openUrl).not.toHaveBeenCalled();
  });

  it('redirects to /setup when reset succeeds', async () => {
    resetSetup.mockResolvedValue(true);
    render(Page);
    screen.getByRole('button', { name: 'Change' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/setup'));
  });

  it('toasts an error when reset fails', async () => {
    resetSetup.mockResolvedValue(false);
    render(Page);
    screen.getByRole('button', { name: 'Change' }).click();
    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('Failed to reset setup')
    );
  });

  it('redirects to /setup when no instance url is configured', async () => {
    setupStatusState.value = {};
    render(Page);
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/setup'));
  });

  it('shows the Disconnected badge while disconnected', () => {
    isConnected.mockReturnValue(false);
    render(Page);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });
});
