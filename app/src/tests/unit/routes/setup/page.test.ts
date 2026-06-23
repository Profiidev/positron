import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const setup = vi.fn();
vi.mock('$lib/commands/setup.svelte', () => ({ setup }));

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

const setupStatusState = { value: undefined as unknown };
vi.mock('$lib/updater/state.svelte', () => ({ setupStatusState }));

const Page = (await import('$routes/setup/+page.svelte')).default;

const okResponse = {
  headers: { get: (k: string) => (k === 'X-Api-Version' ? '1' : null) },
  status: 200
} as unknown as Response;

const submit = async () => {
  const input = screen.getByPlaceholderText('https://positron.example.com');
  await fireEvent.input(input, { target: { value: 'https://example.com' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));
};

// A failed `onsubmit` keeps the user on the stage and swaps the submit button
// For a destructive "Retry" affordance; that is the user-visible error signal.
const expectConnectError = async () =>
  vi.waitFor(() =>
    expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument()
  );

beforeEach(() => {
  setupStatusState.value = undefined;
});
afterEach(() => vi.clearAllMocks());

describe('setup page', () => {
  it('renders the instance url step', () => {
    render(Page);
    expect(
      screen.getByPlaceholderText('https://positron.example.com')
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Confirm' })).toBeInTheDocument();
  });

  it('redirects home when a setup url already exists', async () => {
    setupStatusState.value = { url: 'https://example.com' };
    render(Page);
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/'));
  });

  it('persists the url and routes to /auth on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => okResponse)
    );
    setup.mockResolvedValue(true);
    render(Page);
    await submit();
    await vi.waitFor(() =>
      expect(setup).toHaveBeenCalledWith('https://example.com')
    );
    expect(toast.success).toHaveBeenCalledWith('Positron setup successful.');
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/auth'));
    vi.unstubAllGlobals();
  });

  it('surfaces a connection error when health check fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ headers: { get: () => null }, status: 500 }))
    );
    render(Page);
    await submit();
    await expectConnectError();
    expect(setup).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it('surfaces a connection error when fetch throws', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => {
        throw new Error('network');
      })
    );
    render(Page);
    await submit();
    await expectConnectError();
    expect(setup).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });
});
