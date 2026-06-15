import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { gotoReady } from '$test_helpers/layout';

// No `special_valid` seeded here, so the account auth actions must first pass
// through the "Confirm Access" re-authentication dialog.
test.beforeEach(async ({ context }) => setupSession(context));

test.describe('confirm access gate', () => {
  test('prompts for re-authentication before changing the password', async ({
    page
  }) => {
    await gotoReady(page, '/account/auth');

    await page.getByRole('button', { name: 'Change Password' }).click();

    // The re-auth dialog intercepts the action first.
    await expect(
      page.getByRole('heading', { name: 'Confirm Access' })
    ).toBeVisible();
  });

  test('unlocks the action after a correct password', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await page.getByRole('button', { name: 'Change Password' }).click();
    await page.getByPlaceholder('Password', { exact: true }).fill('mypassword');
    await page.getByRole('button', { name: 'Confirm Access' }).click();

    // Access granted -> the original change-password dialog now opens.
    await expect(
      page.getByPlaceholder('New Password', { exact: true })
    ).toBeVisible();
  });
});
