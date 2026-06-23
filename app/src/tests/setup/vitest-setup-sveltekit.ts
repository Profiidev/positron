import { vi } from 'vitest';
import type * as AppForms from '$app/forms';

// Global stubs for SvelteKit's `$app/*` modules so components built on
// Sveltekit-superforms (which call `beforeNavigate`, subscribe to the `page`
// Store, etc.) can render under jsdom. Individual tests may still override
// `$app/navigation` with their own `vi.mock` to assert navigation calls.

vi.mock('$app/navigation', () => ({
  afterNavigate: vi.fn(),
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

// Superforms calls `applyAction` after a (non-cancelled) SPA submit. The real
// One reaches into SvelteKit's client root component, which doesn't exist under
// Jsdom; stub just that one export while keeping the real `enhance` (which
// Superforms' own enhance builds on).
vi.mock('$app/forms', async (importActual) => {
  const actual = await importActual<typeof AppForms>();
  return { ...actual, applyAction: vi.fn(async () => Promise.resolve()) };
});

vi.mock('$app/state', () => ({
  navigating: { complete: Promise.resolve(), from: null, to: null, type: null },
  page: {
    data: {},
    error: null,
    form: null,
    params: {},
    route: { id: null },
    status: 200,
    url: new URL('http://localhost/')
  },
  updated: { current: false }
}));
