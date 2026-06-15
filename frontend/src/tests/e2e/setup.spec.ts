import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { seedSetupPending } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.describe('first-run setup', () => {
  test.beforeEach(async ({ context }) => seedSetupPending(context));

  test('renders the setup wizard on an un-provisioned instance', async ({
    page
  }) => {
    await gotoReady(page, '/setup');

    await expect(page.getByText('Disclaimer', { exact: true })).toBeVisible();
    await expect(page.getByText('I have read the disclaimer')).toBeVisible();
    // The detected backend is surfaced from the isSetup response.
    await expect(page.getByText('sqlite')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('forces other pages back to the setup wizard', async ({ page }) => {
    await gotoReady(page, '/login');
    await expect(page).toHaveURL(/\/setup/);
  });

  test('walks through both wizard stages to completion', async ({ page }) => {
    await gotoReady(page, '/setup');

    // Stage 1: accept the disclaimer, then advance.
    const disclaimer = page.getByRole('checkbox', {
      name: 'I have read the disclaimer'
    });
    await disclaimer.dispatchEvent('click');
    await expect(disclaimer).toBeChecked();
    await page.getByRole('button', { name: 'Next' }).click();

    // Stage 2: the admin-user form is shown.
    const username = page.getByPlaceholder('Enter username');
    await expect(username).toBeVisible();
    await username.fill('Admin');
    await page.getByPlaceholder('Enter email').fill('admin@example.com');
    await page.getByPlaceholder('Enter password').fill('supersecret');

    await page.getByRole('button', { name: 'Finish' }).click();

    // Completing setup invalidates and navigates to `/`, which (without an auth
    // Cookie) redirects to the login page.
    await expect(page).toHaveURL(/\/login/);
  });

  test('blocks advancing without accepting the disclaimer', async ({
    page
  }) => {
    await gotoReady(page, '/setup');

    await page.getByRole('button', { name: 'Next' }).click();

    // Validation keeps the wizard on stage 1 (admin fields never appear).
    await expect(page.getByPlaceholder('Enter username')).toHaveCount(0);
  });
});

test('redirects away from /setup once provisioned', async ({ page }) => {
  // No mock_setup cookie => isSetup reports a provisioned (and unauthenticated)
  // Instance, so /setup bounces to / and then to /login.
  await gotoReady(page, '/setup');
  await expect(page).toHaveURL(/\/login/);
});
