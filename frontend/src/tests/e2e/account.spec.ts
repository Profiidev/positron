import { expect } from '@playwright/test';
import { HttpResponse } from 'msw';
import * as gen from '$lib/client/msw.gen';
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

  test('saves the oauth confirmation setting', async ({ page }) => {
    await gotoReady(page, '/account/settings');

    await page.getByRole('switch').first().click();
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('General settings saved successfully')
    ).toBeVisible();
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

test.describe('account sessions', () => {
  test('renders active sessions with device and type details', async ({
    page
  }) => {
    await gotoReady(page, '/account/sessions');

    await expect(page.getByRole('heading', { name: 'Sessions' })).toBeVisible();
    // Both fixture sessions render, with their device names and subtitles.
    await expect(page.getByText('MacBook Pro')).toBeVisible();
    await expect(page.getByText('iPhone 15 Pro')).toBeVisible();
    // The current session is badged and the type column distinguishes app from
    // Browser sessions.
    await expect(page.getByText('This device')).toBeVisible();
    await expect(page.getByText('Browser', { exact: true })).toBeVisible();
    await expect(page.getByText('App', { exact: true })).toBeVisible();
    // One non-current session exists, so the bulk-revoke action is enabled.
    await expect(
      page.getByRole('button', { name: 'Revoke all other sessions' })
    ).toBeEnabled();
    await expectNoHorizontalOverflow(page);
  });

  test('renders the empty state with bulk revoke disabled', async ({
    page,
    context
  }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/account/sessions');

    await expect(page.getByRole('heading', { name: 'Sessions' })).toBeVisible();
    await expect(page.getByText('No results.')).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Revoke all other sessions' })
    ).toBeDisabled();
    await expectNoHorizontalOverflow(page);
  });

  test('disables the action for the current session', async ({ page }) => {
    await gotoReady(page, '/account/sessions');

    // The current session shows a locked, disabled action instead of a revoke
    // Button.
    await expect(
      page.getByRole('button', { name: 'Current session' })
    ).toBeDisabled();
    await expect(
      page.getByRole('button', { name: 'Revoke session' })
    ).toBeEnabled();
  });

  test('revokes another session', async ({ page }) => {
    await gotoReady(page, '/account/sessions');

    await page.getByRole('button', { name: 'Revoke session' }).first().click();
    await page.getByRole('button', { exact: true, name: 'Revoke' }).click();

    await expect(page.getByText('Session revoked')).toBeVisible();
  });

  test('surfaces an error when revoking fails', async ({ page, network }) => {
    network.use(
      gen.revokeSessionMswHandler(
        () => new HttpResponse(null, { status: 500 }) as never
      )
    );

    await gotoReady(page, '/account/sessions');

    await page.getByRole('button', { name: 'Revoke session' }).first().click();
    await page.getByRole('button', { exact: true, name: 'Revoke' }).click();

    await expect(page.getByText('Failed to revoke session')).toBeVisible();
    // The dialog stays open and offers a retry after the failed request.
    await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible();
  });

  test('revokes all other sessions', async ({ page }) => {
    await gotoReady(page, '/account/sessions');

    await page
      .getByRole('button', { name: 'Revoke all other sessions' })
      .click();
    await page.getByRole('button', { exact: true, name: 'Revoke all' }).click();

    await expect(page.getByText('Sessions revoked')).toBeVisible();
  });
});

test('navigates between account tabs', async ({ page }) => {
  await gotoReady(page, '/account/general');
  await page.locator('a[href="/account/settings"]').first().click();

  await expect(page).toHaveURL(/\/account\/settings/);
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();

  await page.locator('a[href="/account/sessions"]').first().click();
  await expect(page).toHaveURL(/\/account\/sessions/);
  await expect(page.getByRole('heading', { name: 'Sessions' })).toBeVisible();
});
