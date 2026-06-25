import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const confirmCode = vi.fn();
const logout = vi.fn(async () => true);
vi.mock('$lib/commands/auth.svelte', () => ({ confirmCode, logout }));

const openUrl = vi.fn(async () => {});
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl }));

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

const userInfoState = { value: undefined as unknown };
const userAvatarState = { value: undefined as unknown };
const setupStatusState = { value: undefined as unknown };
vi.mock('$lib/updater/state.svelte.js', () => ({
  setupStatusState,
  userAvatarState,
  userInfoState
}));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte.js', () => ({ isConnected }));

const Page = (await import('$routes/(app)/login/+page.svelte')).default;

const data = (code?: string, redirect?: string) => ({ code, redirect });

beforeEach(() => {
  userInfoState.value = {
    email: 'ada@example.com',
    name: 'Ada Lovelace',
    uuid: 'u1'
  };
  userAvatarState.value = 'blob:avatar';
  setupStatusState.value = { url: 'https://example.com' };
  isConnected.mockReturnValue(true);
});
afterEach(() => vi.clearAllMocks());

describe('login page', () => {
  it('renders the user identity when info is loaded', () => {
    render(Page, { data: data('code-1') });
    expect(screen.getByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('ada@example.com')).toBeInTheDocument();
  });

  it('falls back to skeletons while the user is loading', () => {
    userInfoState.value = undefined;
    render(Page, { data: data('code-1') });
    expect(screen.queryByText('Ada Lovelace')).not.toBeInTheDocument();
  });

  it('confirms login, follows the allowed redirect, and goes home', async () => {
    confirmCode.mockResolvedValue(true);
    render(Page, { data: data('code-1', 'https://example.com/app') });
    screen.getByRole('button', { name: 'Confirm' }).click();
    await vi.waitFor(() =>
      expect(toast.success).toHaveBeenCalledWith(
        'Login confirmed successfully.'
      )
    );
    expect(confirmCode).toHaveBeenCalledWith('code-1');
    expect(openUrl).toHaveBeenCalledWith('https://example.com/app');
    expect(goto).toHaveBeenCalledWith('/');
  });

  it('does not follow a cross-origin redirect but still goes home', async () => {
    confirmCode.mockResolvedValue(true);
    render(Page, { data: data('code-1', 'https://evil.example.net/app') });
    screen.getByRole('button', { name: 'Confirm' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/'));
    expect(openUrl).not.toHaveBeenCalled();
  });

  it('toasts an error when confirmation fails', async () => {
    confirmCode.mockResolvedValue(undefined);
    render(Page, { data: data('code-1') });
    screen.getByRole('button', { name: 'Confirm' }).click();
    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('Failed to confirm login.')
    );
    expect(goto).not.toHaveBeenCalled();
  });

  it('does nothing when there is no code to confirm', async () => {
    render(Page, { data: data(undefined) });
    screen.getByRole('button', { name: 'Confirm' }).click();
    await Promise.resolve();
    expect(confirmCode).not.toHaveBeenCalled();
  });

  it('logs out and routes to /auth on Change', async () => {
    render(Page, { data: data('code-1') });
    screen.getByRole('button', { name: 'Change' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/auth'));
    expect(logout).toHaveBeenCalled();
  });

  it('goes home on Cancel', () => {
    render(Page, { data: data('code-1') });
    screen.getByRole('button', { name: 'Cancel' }).click();
    expect(goto).toHaveBeenCalledWith('/');
  });

  it('shows the Disconnected badge while disconnected', () => {
    isConnected.mockReturnValue(false);
    render(Page, { data: data('code-1') });
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });
});
