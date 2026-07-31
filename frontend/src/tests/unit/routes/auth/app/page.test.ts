import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const requestAppCode = vi.fn();
const logout = vi.fn();
vi.mock('$lib/client', () => ({ logout, requestAppCode }));

const toast = { error: vi.fn(), success: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

const Page = (await import('$routes/auth/app/+page.svelte')).default;

const user = {
  email: 'ada@example.com',
  name: 'Ada',
  permissions: [],
  totp_enabled: false,
  uuid: 'u1'
};

const data = (challenge: string | null = 'chal') =>
  ({
    auth: { authType: 'app', challenge },
    user: Promise.resolve(user)
  }) as never;

const okResponse = (code: string) => ({
  data: { code },
  response: { status: 200 }
});

beforeEach(() => {
  Object.defineProperty(window, 'location', {
    configurable: true,
    value: { href: '' }
  });
  vi.spyOn(window, 'close').mockImplementation(() => {});
});

afterEach(() => {
  vi.restoreAllMocks();
  Object.defineProperty(navigator, 'userAgentData', {
    configurable: true,
    value: undefined
  });
});

describe('auth/app page', () => {
  it('shows the account once it resolves', async () => {
    render(Page, { data: data() });
    await vi.waitFor(() => expect(screen.getByText('Ada')).toBeInTheDocument());
    expect(screen.getByText('ada@example.com')).toBeInTheDocument();
  });

  it('errors without redirecting to the app scheme when the challenge is missing', async () => {
    render(Page, { data: data(null) });
    await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));
    expect(toast.error).toHaveBeenCalledWith(
      'There was an error while login in'
    );
    expect(requestAppCode).not.toHaveBeenCalled();
  });

  it('redirects to the app scheme without closing the window on desktop', async () => {
    requestAppCode.mockResolvedValue(okResponse('abc123'));
    render(Page, { data: data() });

    await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));

    await vi.waitFor(() =>
      expect(window.location.href).toBe('positron://auth?code=abc123')
    );
    expect(window.close).not.toHaveBeenCalled();
  });

  it('redirects to the app scheme and closes the window on mobile', async () => {
    Object.defineProperty(navigator, 'userAgentData', {
      configurable: true,
      value: { mobile: true }
    });
    requestAppCode.mockResolvedValue(okResponse('abc123'));
    render(Page, { data: data() });

    await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));

    await vi.waitFor(() =>
      expect(window.location.href).toBe('positron://auth?code=abc123')
    );
    expect(window.close).toHaveBeenCalled();
  });

  it('toasts an error when the request fails', async () => {
    requestAppCode.mockResolvedValue({
      data: undefined,
      response: { status: 500 }
    });
    render(Page, { data: data() });

    await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));

    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith(
        'There was an error while login in'
      )
    );
    expect(window.close).not.toHaveBeenCalled();
  });

  it('cancels back to the home page', async () => {
    render(Page, { data: data() });
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(goto).toHaveBeenCalledWith('/');
  });
});
