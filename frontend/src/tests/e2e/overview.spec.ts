import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => {
  await setupSession(context);
});

test('overview page renders inside the app shell', async ({ page }) => {
  await gotoReady(page, '/');

  await expect(page.getByRole('main').getByText('Overview')).toBeVisible();
  await expectNoHorizontalOverflow(page);
});

test('sidebar wires up the navigation targets', async ({ page }, testInfo) => {
  test.skip(
    (testInfo.project.use.viewport?.width ?? 0) < 1024,
    'sidebar is behind a trigger on mobile'
  );

  await gotoReady(page, '/');

  for (const href of [
    '/notes',
    '/apod',
    '/oauth-client',
    '/oauth-scope',
    '/oauth-policy',
    '/users',
    '/groups',
    '/settings'
  ]) {
    // oxlint-disable-next-line no-await-in-loop
    await expect(page.locator(`a[href="${href}"]`).first()).toBeAttached();
  }
});

test('navigating to notes from the sidebar works on desktop', async ({
  page
}, testInfo) => {
  test.skip(
    (testInfo.project.use.viewport?.width ?? 0) < 1024,
    'sidebar is behind a trigger on mobile'
  );

  await gotoReady(page, '/');
  await page.locator('a[href="/notes"]').first().click();

  await expect(page).toHaveURL(/\/notes/);
  await expect(page.getByRole('heading', { name: 'Notes' })).toBeVisible();
});
