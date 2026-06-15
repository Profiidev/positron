import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e_fixture';
import { seedSetupPending } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.describe('first-run setup', () => {
  test.beforeEach(async ({ context }) => seedSetupPending(context));

  test('renders the setup wizard on an un-provisioned instance', async ({
    page
  }) => {
    await gotoReady(page, '/setup');

    await expect(page.getByText('Disclaimer', { exact: true })).toBeVisible();
    await expect(
      page.getByText('I have read the disclaimer')
    ).toBeVisible();
    // the detected backend is surfaced from the isSetup response.
    await expect(page.getByText('sqlite')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('forces other pages back to the setup wizard', async ({ page }) => {
    await gotoReady(page, '/login');
    await expect(page).toHaveURL(/\/setup/);
  });
});

test('redirects away from /setup once provisioned', async ({ page }) => {
  // no mock_setup cookie => isSetup reports a provisioned (and unauthenticated)
  // instance, so /setup bounces to / and then to /login.
  await gotoReady(page, '/setup');
  await expect(page).toHaveURL(/\/login/);
});
