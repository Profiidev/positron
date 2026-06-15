import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test('redirects /account to the general tab', async ({ page }) => {
  await gotoReady(page, '/account');
  await expect(page).toHaveURL(/\/account\/general/);
});

test.describe('account general', () => {
  test('prefills the username from the account', async ({ page }) => {
    await gotoReady(page, '/account/general');

    await expect(page.getByRole('heading', { name: 'Account' })).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'General Settings' })
    ).toBeVisible();
    await expect(page.getByPlaceholder('Enter your username')).toHaveValue(
      'Ada Admin'
    );
    await expectNoHorizontalOverflow(page);
  });

  test('saves general settings', async ({ page }) => {
    await gotoReady(page, '/account/general');

    const username = page.getByPlaceholder('Enter your username');
    await expect(username).toHaveValue('Ada Admin');
    await username.fill('Ada Lovelace');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('General settings saved successfully')
    ).toBeVisible();
  });
});

test.describe('account settings', () => {
  test('renders the oauth confirmation toggle', async ({ page }) => {
    await gotoReady(page, '/account/settings');

    await expect(
      page.getByText('Skip confirmation for OAuth logins')
    ).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });
});

test.describe('account authentication', () => {
  test('renders the authentication methods', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await expect(
      page.getByRole('heading', { name: 'Authentication' })
    ).toBeVisible();
    await expect(page.getByText('Other 2FA Methods')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });
});

test('navigates between account tabs', async ({ page }) => {
  await gotoReady(page, '/account/general');
  await page.locator('a[href="/account/settings"]').first().click();

  await expect(page).toHaveURL(/\/account\/settings/);
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
});
