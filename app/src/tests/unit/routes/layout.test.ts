import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';
import { page } from '$app/state';

const startListener = vi.fn(async () => () => {});
const setOnline = vi.fn(async () => true);
vi.mock('$lib/updater/updater.svelte', () => ({ setOnline, startListener }));

const setupStatusState = { value: undefined as unknown };
const authStatusState = { value: undefined as unknown };
vi.mock('$lib/updater/state.svelte', () => ({
  authStatusState,
  setupStatusState
}));

const Layout = (await import('$routes/+layout.svelte')).default;

const children = createRawSnippet(() => ({
  render: () => '<div data-testid="child">child</div>'
}));
const renderLayout = () => render(Layout, { children });

beforeEach(() => {
  page.route.id = '/';
  setupStatusState.value = { url: 'https://example.com' };
  authStatusState.value = true;
});
afterEach(() => vi.clearAllMocks());

describe('root layout', () => {
  it('renders children and starts the updater listener', () => {
    renderLayout();
    expect(screen.getByTestId('child')).toBeInTheDocument();
    expect(startListener).toHaveBeenCalled();
  });

  it('redirects to /setup when the instance is not configured', async () => {
    setupStatusState.value = {};
    renderLayout();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/setup'));
  });

  it('does not redirect while setup status is still loading (null)', async () => {
    // oxlint-disable-next-line no-null
    setupStatusState.value = null;
    authStatusState.value = null;
    renderLayout();
    await Promise.resolve();
    expect(goto).not.toHaveBeenCalled();
  });

  it('does not redirect to /setup while already on /setup', () => {
    setupStatusState.value = {};
    page.route.id = '/setup';
    renderLayout();
    expect(goto).not.toHaveBeenCalledWith('/setup');
  });

  it('redirects to /auth when unauthenticated', async () => {
    authStatusState.value = false;
    renderLayout();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/auth'));
  });

  it('does not redirect to /auth while on /auth or /setup', () => {
    authStatusState.value = false;
    page.route.id = '/auth';
    renderLayout();
    expect(goto).not.toHaveBeenCalledWith('/auth');
  });

  it('does not redirect to /auth while auth status is undefined', () => {
    authStatusState.value = undefined;
    renderLayout();
    expect(goto).not.toHaveBeenCalledWith('/auth');
  });
});
