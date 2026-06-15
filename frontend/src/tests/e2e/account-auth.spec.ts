import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { seedSpecialAccess, setupSession } from '$test_helpers/session';
import { gotoReady } from '$test_helpers/layout';

// `special_valid` is seeded so the re-auth ("Confirm Access") dialog is skipped
// And the account auth actions open directly.
test.beforeEach(async ({ context }) => {
  await setupSession(context);
  await seedSpecialAccess(context);
});

test.describe('password change', () => {
  test('changes the password with matching values', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await page.getByRole('button', { name: 'Change Password' }).click();
    await page
      .getByPlaceholder('New Password', { exact: true })
      .fill('newsecret');
    await page.getByPlaceholder('Confirm New Password').fill('newsecret');
    await page.getByRole('button', { name: 'Change Password' }).last().click();

    await expect(
      page.getByText('Password was changed successfully')
    ).toBeVisible();
  });

  test('rejects mismatched passwords', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await page.getByRole('button', { name: 'Change Password' }).click();
    await page
      .getByPlaceholder('New Password', { exact: true })
      .fill('newsecret');
    await page.getByPlaceholder('Confirm New Password').fill('different');
    await page.getByRole('button', { name: 'Change Password' }).last().click();

    await expect(page.getByText('Passwords are not equal')).toBeVisible();
  });
});

test.describe('passkeys', () => {
  test('renders the existing passkey', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await expect(page.getByText('YubiKey')).toBeVisible();
  });

  test('renames a passkey', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    // The pencil button on the passkey row opens the rename dialog.
    await page
      .getByText('YubiKey')
      .locator('xpath=ancestor::div[contains(@class,"flex items-center")][1]')
      .getByRole('button')
      .first()
      .click();

    const nameInput = page.getByPlaceholder('Name');
    await nameInput.fill('Renamed Key');
    await page.getByRole('button', { name: 'Confirm' }).click();

    await expect(page.getByText('Edit successful')).toBeVisible();
  });

  test('deletes a passkey', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await page
      .getByText('YubiKey')
      .locator('xpath=ancestor::div[contains(@class,"flex items-center")][1]')
      .getByRole('button')
      .last()
      .click();

    await page.getByRole('button', { name: 'Confirm' }).click();

    await expect(page.getByText('Deletion successful')).toBeVisible();
  });
});

test.describe('totp 2fa', () => {
  test('shows TOTP as disabled and opens the add dialog', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    await expect(page.getByText('TOTP')).toBeVisible();
    await expect(page.getByText('Disabled')).toBeVisible();
  });

  test('adds a TOTP authenticator', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    // Exact name avoids matching the "Add Passkey" button.
    await page.getByRole('button', { exact: true, name: 'Add' }).click();
    await expect(page.getByRole('heading', { name: 'Add TOTP' })).toBeVisible();

    // The OTP field auto-focuses inside the dialog; type the 6-digit code.
    await page.keyboard.type('123456');
    await page
      .getByRole('dialog')
      .getByRole('button', { exact: true, name: 'Add' })
      .click();

    await expect(
      page.getByText('TOTP was added successfully to your account')
    ).toBeVisible();
  });
});

test.describe('email change', () => {
  test('advances to the verification-code step', async ({ page }) => {
    await gotoReady(page, '/account/auth');

    // The trigger is wrapped in a tooltip (two matching buttons); the inner one
    // Carries the click handler that opens the dialog.
    await page.getByRole('button', { name: 'Change Email' }).last().click();

    const dialog = page.getByRole('dialog');
    await dialog
      .getByPlaceholder('mail@example.com')
      .fill('new.mail@example.com');
    await dialog.getByRole('button', { name: 'Change Email' }).click();

    // After starting the change, the dialog asks for the codes from both inboxes.
    await expect(page.getByText(/Code from old Email/)).toBeVisible();
    await expect(page.getByText(/Code from new Email/)).toBeVisible();
  });
});
