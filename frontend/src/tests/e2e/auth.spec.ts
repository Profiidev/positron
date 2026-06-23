import { type Page, expect } from '@playwright/test';
import { HttpResponse } from 'msw';
import * as gen from '$lib/client/msw.gen';
import { test } from '$test_helpers/e2e-fixture';
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
      page.getByRole('button', { exact: true, name: 'Login' })
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Passkey' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'App Login' })).toBeVisible();
    // Mail is enabled in the mock config, so the reset link is shown.
    await expect(
      page.getByRole('link', { name: 'Forgot your password?' })
    ).toBeVisible();

    await expectNoHorizontalOverflow(page);
  });

  test('does not submit with an invalid email', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByPlaceholder('mail@example.com').fill('not-an-email');
    await page.getByPlaceholder('Your password').fill('secret');
    await page.getByRole('button', { exact: true, name: 'Login' }).click();

    // Invalid input is rejected, so the form stays on the login page.
    await expect(page).toHaveURL(/\/login/);
  });

  test('does not submit without a password', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByRole('button', { exact: true, name: 'Login' }).click();

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

  test('redirects an authenticated user to the redirect target', async ({
    context,
    page
  }) => {
    await setupSession(context);
    await gotoReady(page, '/login?redirect=%2Fusers');

    await expect(page).toHaveURL(/\/users$/);
  });

  test('ignores an unsafe redirect param and lands on the app root', async ({
    context,
    page,
    baseURL
  }) => {
    await setupSession(context);
    // An open-redirect attempt must never leave the app; it falls back to '/'.
    await gotoReady(page, '/login?redirect=%2F%2Fevil.com');

    // Stayed on the app origin (not evil.com) and off /login.
    expect(new URL(page.url()).host).toBe(new URL(baseURL!).host);
    await expect(page).not.toHaveURL(/\/login/);
  });

  test('returns to the originally requested URL after login', async ({
    page,
    network
  }) => {
    network.use(
      gen.passwordAuthenticateMswHandler(() =>
        HttpResponse.json(
          { user: 'user-uuid' },
          {
            headers: {
              'Set-Cookie': 'centaurus_jwt=e2e-token; Path=/'
            }
          }
        )
      )
    );

    await gotoReady(page, '/users');
    await expect(page).toHaveURL(/\/login\?redirect=%2Fusers/);

    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByPlaceholder('Your password').fill('secret');
    await page.getByRole('button', { exact: true, name: 'Login' }).click();

    await expect(page).toHaveURL(/\/users$/);
  });

  test('attaches session metadata to the password login request', async ({
    page,
    network
  }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let body: any = undefined;
    network.use(
      gen.passwordAuthenticateMswHandler(async ({ request }) => {
        body = await request.json();
        return HttpResponse.json(
          { user: 'user-uuid' },
          { headers: { 'Set-Cookie': 'centaurus_jwt=e2e-token; Path=/' } }
        );
      })
    );

    await gotoReady(page, '/login');
    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByPlaceholder('Your password').fill('secret');
    await page.getByRole('button', { exact: true, name: 'Login' }).click();

    // `getSessionMeta()` is spread into the auth body so the backend can record
    // The device behind each session.
    await expect.poll(() => body).toBeTruthy();
    expect(body).toMatchObject({
      application: expect.any(String),
      name: expect.any(String),
      operating_system: expect.any(String)
    });
  });
});

test.describe('totp login', () => {
  // A password login for a 2FA-enabled account succeeds but returns no `user`,
  // Which flips the form into the TOTP step instead of completing the login.
  const totpRequiredAuth = () =>
    gen.passwordAuthenticateMswHandler(() => HttpResponse.json({}));

  const submitPassword = async (page: Page) => {
    await page.getByPlaceholder('mail@example.com').fill('user@example.com');
    await page.getByPlaceholder('Your password').fill('secret');
    await page.getByRole('button', { exact: true, name: 'Login' }).click();
  };

  test('prompts for the TOTP code after a 2FA password login', async ({
    page,
    network
  }) => {
    network.use(totpRequiredAuth());

    await gotoReady(page, '/login');
    await submitPassword(page);

    // The form swaps to the 6-digit prompt; the email/password inputs are gone.
    await expect(
      page.getByText('Enter the 6-digit code from your authenticator app')
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Continue' })).toBeVisible();
    await expect(page.getByPlaceholder('mail@example.com')).toHaveCount(0);

    await expectNoHorizontalOverflow(page);
  });

  test('completes the login after a valid TOTP code', async ({
    page,
    network
  }) => {
    network.use(
      totpRequiredAuth(),
      gen.totpConfirmMswHandler(() =>
        HttpResponse.json(
          { user: 'user-uuid' },
          { headers: { 'Set-Cookie': 'centaurus_jwt=e2e-token; Path=/' } }
        )
      )
    );

    await gotoReady(page, '/login?redirect=%2Fusers');
    await submitPassword(page);

    await expect(
      page.getByText('Enter the 6-digit code from your authenticator app')
    ).toBeVisible();

    // The OTP field auto-focuses; type the 6-digit code and confirm.
    await page.keyboard.type('123456');
    await page.getByRole('button', { name: 'Continue' }).click();

    await expect(page).toHaveURL(/\/users$/);
  });

  test('shows an error for an invalid TOTP code', async ({ page, network }) => {
    network.use(
      totpRequiredAuth(),
      gen.totpConfirmMswHandler(() => HttpResponse.json({}, { status: 401 }))
    );

    await gotoReady(page, '/login');
    await submitPassword(page);

    await expect(
      page.getByText('Enter the 6-digit code from your authenticator app')
    ).toBeVisible();

    await page.keyboard.type('123456');
    await page.getByRole('button', { name: 'Continue' }).click();

    await expect(page.getByText('Invalid code.')).toBeVisible();
    await expect(page).toHaveURL(/\/login/);
  });

  test('does not submit a code shorter than 6 digits', async ({
    page,
    network
  }) => {
    network.use(totpRequiredAuth());

    await gotoReady(page, '/login');
    await submitPassword(page);

    await expect(
      page.getByText('Enter the 6-digit code from your authenticator app')
    ).toBeVisible();

    await page.keyboard.type('123');
    await page.getByRole('button', { name: 'Continue' }).click();

    await expect(page.getByText('TOTP code must be 6 digits')).toBeVisible();
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
      .getByRole('button', { exact: true, name: 'Reset Password' })
      .click();

    await expect(page.getByText('Passwords do not match')).toBeVisible();
    await expect(page).toHaveURL(/\/password\/reset/);
  });

  test('enforces a minimum password length', async ({ page }) => {
    await gotoReady(page, '/password/reset?token=abc123');

    await page.getByPlaceholder('Enter your new password').fill('123');
    await page.getByPlaceholder('Confirm your new password').fill('123');
    await page
      .getByRole('button', { exact: true, name: 'Reset Password' })
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
      const redirect = encodeURIComponent(path);
      await expect(page).toHaveURL(
        new RegExp(
          `/login\\?redirect=${redirect.replace(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`)}`
        )
      );
    });
  }
});
