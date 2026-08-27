import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const toastError = vi.fn();
vi.mock('@profidev/pleiades/components/util/general', () => ({
  toast: { error: toastError, success: vi.fn() }
}));

// The page reads the error param via `afterNavigate` (not `$effect`) so the
// Cleanup can't run before SvelteKit finishes hydration; override the global
// No-op stub so it actually invokes the callback like a real navigation would.
vi.mock('$app/navigation', () => ({
  afterNavigate: (fn: () => void) => fn(),
  beforeNavigate: vi.fn(),
  disableScrollHandling: vi.fn(),
  goto: vi.fn(async () => Promise.resolve()),
  invalidate: vi.fn(async () => Promise.resolve()),
  invalidateAll: vi.fn(async () => Promise.resolve()),
  onNavigate: vi.fn(),
  preloadCode: vi.fn(async () => Promise.resolve()),
  preloadData: vi.fn(async () => Promise.resolve()),
  pushState: vi.fn(),
  replaceState: vi.fn()
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
