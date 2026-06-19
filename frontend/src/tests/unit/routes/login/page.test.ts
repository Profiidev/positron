import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const toastError = vi.fn();
vi.mock('@profidev/pleiades/components/util/general', () => ({
  toast: { error: toastError, success: vi.fn() }
}));

const Page = (await import('$routes/login/+page.svelte')).default;

const data = (error?: string, redirectTo = '/') =>
  ({
    auth: { authType: null, challenge: null },
    config: Promise.resolve({ mail_enabled: false }),
    error,
    oauthOptions: { code: null, name: null },
    redirectTo
  }) as never;

afterEach(() => toastError.mockClear());

describe('login page', () => {
  it('renders the login form fields', () => {
    render(Page, { data: data() });
    expect(screen.getByText('Email')).toBeInTheDocument();
    expect(screen.getByText('Password')).toBeInTheDocument();
  });

  it.each([
    ['missing_code', 'SSO login failed: Missing authorization code.'],
    ['oidc_not_configured', 'SSO login failed: OIDC is not configured.'],
    ['not_found', 'User not found.'],
    ['weird', 'SSO login failed: weird']
  ])('maps the %s error to a toast', async (code, message) => {
    render(Page, { data: data(code) });
    await vi.waitFor(() => expect(toastError).toHaveBeenCalledWith(message));
  });

  it('shows no error toast when there is no error', () => {
    render(Page, { data: data() });
    expect(toastError).not.toHaveBeenCalled();
  });
});
