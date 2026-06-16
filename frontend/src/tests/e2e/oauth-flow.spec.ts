import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';
import { setupSession } from '$test_helpers/session';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('oauth authorization confirm', () => {
  test('asks the user to confirm logging in to the client', async ({
    page
  }) => {
    await gotoReady(page, '/oauth?code=auth-code&name=Demo%20App');

    await expect(page.getByText('Log in to Demo App')).toBeVisible();
    await expect(page.getByText('Ada Admin')).toBeVisible();
    await expect(page.getByText('admin@example.com')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Confirm' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Change' })).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('cancelling returns to Positron (and the login guard)', async ({
    page
  }) => {
    await gotoReady(page, '/oauth?code=auth-code&name=Demo%20App');
    await page.getByRole('button', { name: 'Cancel' }).click();

    // Cancel navigates to `/`, which the unauthenticated guard bounces to login.
    await expect(page).toHaveURL(/\//);
  });
});

test.describe('oauth logout confirm', () => {
  test('offers to return to the client or to Positron', async ({ page }) => {
    await gotoReady(
      page,
      '/oauth/logout?url=https://demo.example&name=Demo%20App'
    );

    await expect(page.getByText('Logged out of Demo App')).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'To Positron' })
    ).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Log back in' })
    ).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('returns to Positron from the logout screen', async ({ page }) => {
    await gotoReady(
      page,
      '/oauth/logout?url=https://demo.example&name=Demo%20App'
    );
    await page.getByRole('button', { name: 'To Positron' }).click();

    await expect(page).toHaveURL(/\//);
  });
});

test.describe('app login confirm', () => {
  test('asks the user to confirm logging in to the app', async ({ page }) => {
    await gotoReady(page, '/auth/app?challenge=challenge-token');

    await expect(page.getByText('Log in to Positron App')).toBeVisible();
    await expect(page.getByText('Ada Admin')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Confirm' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('cancelling the app login returns to the login guard', async ({
    page
  }) => {
    await gotoReady(page, '/auth/app?challenge=challenge-token');
    await page.getByRole('button', { name: 'Cancel' }).click();

    await expect(page).toHaveURL(/\//);
  });
});
