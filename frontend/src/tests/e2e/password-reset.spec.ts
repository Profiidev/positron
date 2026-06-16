import { expect } from '@playwright/test';
import { HttpResponse } from 'msw';
import * as gen from '$lib/client/msw.gen';
import { test } from '$test_helpers/e2e-fixture';
import { gotoReady } from '$test_helpers/layout';

// Token as it would appear in the link a user clicks from the password-reset
// email: `https://<app>/password/reset?token=<token>`.
const RESET_TOKEN = 'email-reset-token';

test.describe('password reset from an email link', () => {
  test('lets a logged-out user open the reset link and set a new password', async ({
    page,
    network
  }) => {
    // A user clicking the reset link from their inbox is NOT logged in, so the
    // backend's token check fails. The default e2e catch-all returns `{}` for
    // `test_token` (which leaves `valid` undefined and hides the redirect), so
    // we model the real, logged-out response explicitly.
    network.use(
      gen.testTokenMswHandler(() =>
        HttpResponse.json({ valid: false, exp_short: false })
      )
    );

    await gotoReady(page, `/password/reset?token=${RESET_TOKEN}`);

    // BUG: the root layout's `onMount` redirects any session with an invalid
    // token to `/login`, with no exemption for `/password/reset`. So the user
    // never reaches the reset form and these assertions currently fail.
    await expect(page).toHaveURL(/\/password\/reset/);
    await expect(
      page.getByText('Enter your new password below to reset your password')
    ).toBeVisible();

    // The token from the email link is prefilled from the query param.
    await expect(page.getByPlaceholder('Enter your token')).toHaveValue(
      RESET_TOKEN
    );

    await page
      .getByPlaceholder('Enter your new password')
      .fill('new-password-123');
    await page
      .getByPlaceholder('Confirm your new password')
      .fill('new-password-123');

    await page
      .getByRole('button', { name: 'Reset Password', exact: true })
      .click();

    await expect(
      page.getByText('Password has been reset successfully')
    ).toBeVisible();
  });
});
