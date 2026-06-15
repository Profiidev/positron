import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e_fixture';
import { setupSession } from '$test_helpers/session';

test('redirects to /login when unauthenticated', async ({ page }) => {
  await page.goto('/groups');
  await expect(page).toHaveURL(/\/login/);
});

test('groups page lists groups (default scenario)', async ({
  context,
  page
}) => {
  await setupSession(context);
  await page.goto('/groups');

  await expect(page.getByText('Groups', { exact: true })).toBeVisible();
});
