import type { Locator, Page } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectPageFits } from '$test_helpers/layout';

/**
 * Systematic horizontal-overflow sweep. Playwright runs every test across the
 * desktop (chromium/firefox/webkit) and mobile (Pixel 8, iPhone 12) projects
 * from `playwright.config.ts`, so each entry below is checked on every viewport.
 */
const pages: { path: string; ready: (page: Page) => Locator }[] = [
  { path: '/', ready: (p) => p.getByRole('main').getByText('Overview') },

  { path: '/notes', ready: (p) => p.getByRole('heading', { name: 'Notes' }) },
  {
    path: '/notes/create',
    ready: (p) => p.getByPlaceholder('Enter note title')
  },
  { path: '/notes/note-1', ready: (p) => p.getByPlaceholder('Note title') },
  {
    path: '/notes/note-1/snapshot-1',
    ready: (p) => p.getByRole('button', { name: 'Restore' })
  },

  { path: '/apod', ready: (p) => p.getByRole('tab', { name: 'Today' }) },
  { path: '/apod/list', ready: (p) => p.getByRole('tab', { name: 'Library' }) },

  {
    path: '/oauth-client',
    ready: (p) => p.getByRole('heading', { name: 'OAuth / Oidc Clients' })
  },
  {
    path: '/oauth-client/create',
    ready: (p) => p.getByPlaceholder('Enter client name')
  },
  {
    path: '/oauth-client/client-1',
    ready: (p) => p.getByRole('heading', { name: /Client:/ })
  },

  {
    path: '/oauth-scope',
    ready: (p) => p.getByRole('heading', { name: 'OAuth / Oidc Scopes' })
  },
  {
    path: '/oauth-scope/create',
    ready: (p) => p.getByPlaceholder('Enter scope name')
  },
  {
    path: '/oauth-scope/scope-1',
    ready: (p) => p.getByRole('heading', { name: /Scope:/ })
  },

  {
    path: '/oauth-policy',
    ready: (p) => p.getByRole('heading', { name: 'OAuth / Oidc Policies' })
  },
  {
    path: '/oauth-policy/create',
    ready: (p) => p.getByPlaceholder('Enter policy name')
  },
  {
    path: '/oauth-policy/policy-1',
    ready: (p) => p.getByRole('heading', { name: /Policy:/ })
  },

  { path: '/users', ready: (p) => p.getByRole('heading', { name: 'Users' }) },
  {
    path: '/users/create',
    ready: (p) => p.getByPlaceholder('Enter user name')
  },
  {
    path: '/users/user-1',
    ready: (p) => p.getByRole('heading', { name: /User:/ })
  },

  { path: '/groups', ready: (p) => p.getByRole('heading', { name: 'Groups' }) },
  {
    path: '/groups/create',
    ready: (p) => p.getByPlaceholder('Enter group name')
  },
  {
    path: '/groups/group-admins',
    ready: (p) => p.getByRole('heading', { name: /Group:/ })
  },

  {
    path: '/settings/mail',
    ready: (p) => p.getByRole('heading', { name: 'Mail Settings' })
  },
  {
    path: '/account/general',
    ready: (p) => p.getByRole('heading', { name: 'General Settings' })
  },
  {
    path: '/account/settings',
    ready: (p) => p.getByText('Skip confirmation for OAuth logins')
  },
  {
    path: '/account/sessions',
    ready: (p) => p.getByRole('heading', { name: 'Sessions' })
  },
  {
    path: '/account/auth',
    ready: (p) => p.getByRole('heading', { name: 'Authentication' })
  }
];

test.describe('no horizontal overflow', () => {
  test.beforeEach(async ({ context }) => setupSession(context));

  for (const { path, ready } of pages) {
    test(`${path} fits its viewport`, async ({ page }) => {
      await expectPageFits(page, path, ready(page));
    });
  }
});
