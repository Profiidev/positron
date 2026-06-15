import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e_fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.describe('login page', () => {
  test('renders the login form with all entry points', async ({ page }) => {
    await gotoReady(page, '/login');

    await expect(
      page.getByText('Enter your login details below to login')
    ).toBeVisible();
    await expect(page.getByPlaceholder('mail@example.com')).toBeVisible();
    await expect(page.getByPlaceholder('Your password')).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Login', exact: true })
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Passkey' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'App Login' })).toBeVisible();
    // mail is enabled in the mock config, so the reset link is shown.
    await expect(
      page.getByRole('link', { name: 'Forgot your password?' })
    ).toBeVisible();

    await expectNoHorizontalOverflow(page);
  });

  test('does not submit with an invalid email', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByPlaceholder('mail@example.com').fill('not-an-email');
    await page.getByPlaceholder('Your password').fill('secret');
    await page.getByRole('button', { name: 'Login', exact: true }).click();

    // invalid input is rejected, so the form stays on the login page.
    await expect(page).toHaveURL(/\/login/);
  });

  test('does not submit without a password', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByRole('button', { name: 'Login', exact: true }).click();

    await expect(page).toHaveURL(/\/login/);
  });

  test('forgot-password link navigates to the reset request page', async ({
    page
  }) => {
    await gotoReady(page, '/login');
    await page.getByRole('link', { name: 'Forgot your password?' }).click();

    await expect(page).toHaveURL(/\/password\/forgot/);
    await expect(page.getByText('Forgot Password')).toBeVisible();
  });

  test('redirects an authenticated user away from /login', async ({
    context,
    page
  }) => {
    await setupSession(context);
    await gotoReady(page, '/login');

    await expect(page).not.toHaveURL(/\/login/);
  });
});

test.describe('forgot password', () => {
  test('sends a reset link on success', async ({ page }) => {
    await gotoReady(page, '/password/forgot');

    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByRole('button', { name: 'Send Reset Link' }).click();

    await expect(
      page.getByText('Reset link sent to your email address.')
    ).toBeVisible();
  });

  test('does not submit an invalid email', async ({ page }) => {
    await gotoReady(page, '/password/forgot');

    await page.getByPlaceholder('mail@example.com').fill('nope');
    await page.getByRole('button', { name: 'Send Reset Link' }).click();

    await expect(page).toHaveURL(/\/password\/forgot/);
  });
});

test.describe('reset password', () => {
  test('prefills the token from the query string', async ({ page }) => {
    await gotoReady(page, '/password/reset?token=abc123');

    await expect(page.getByPlaceholder('Enter your token')).toHaveValue(
      'abc123'
    );
  });

  test('rejects mismatched passwords', async ({ page }) => {
    await gotoReady(page, '/password/reset?token=abc123');

    await page.getByPlaceholder('Enter your new password').fill('supersecret');
    await page
      .getByPlaceholder('Confirm your new password')
      .fill('different-secret');
    await page
      .getByRole('button', { name: 'Reset Password', exact: true })
      .click();

    await expect(page.getByText('Passwords do not match')).toBeVisible();
    await expect(page).toHaveURL(/\/password\/reset/);
  });

  test('enforces a minimum password length', async ({ page }) => {
    await gotoReady(page, '/password/reset?token=abc123');

    await page.getByPlaceholder('Enter your new password').fill('123');
    await page.getByPlaceholder('Confirm your new password').fill('123');
    await page
      .getByRole('button', { name: 'Reset Password', exact: true })
      .click();

    await expect(
      page.getByText('Password must be at least 6 characters long')
    ).toBeVisible();
  });
});

test.describe('auth guards', () => {
  for (const path of ['/', '/users', '/groups', '/notes', '/account']) {
    test(`redirects ${path} to /login when unauthenticated`, async ({
      page
    }) => {
      await gotoReady(page, path);
      await expect(page).toHaveURL(/\/login/);
    });
  }
});
