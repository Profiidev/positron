import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e_fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('oauth clients', () => {
  test('lists clients in the default scenario', async ({ page }) => {
    await gotoReady(page, '/oauth-client');

    await expect(
      page.getByRole('heading', { name: 'OAuth / Oidc Clients' })
    ).toBeVisible();
    await expect(page.getByText('Dashboard App')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no clients', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/oauth-client');

    await expect(page.getByText('No results.')).toBeVisible();
  });

  test('creates a client and navigates to its detail page', async ({
    page
  }) => {
    await gotoReady(page, '/oauth-client/create');

    await page.getByPlaceholder('Enter client name').fill('My App');
    await page
      .getByPlaceholder('https://example.com/callback')
      .fill('https://app.example/callback');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-client\/client-new/);
  });

  test('rejects a client with no name', async ({ page }) => {
    await gotoReady(page, '/oauth-client/create');

    await page
      .getByPlaceholder('https://example.com/callback')
      .fill('https://app.example/callback');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-client\/create/);
  });

  test('renders client detail and deletes the client', async ({ page }) => {
    await gotoReady(page, '/oauth-client/client-1');

    await expect(page.getByRole('heading', { name: /Client:/ })).toContainText(
      'Dashboard App'
    );

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the client Dashboard App?')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page).toHaveURL(/\/oauth-client$/);
  });
});

test.describe('oauth scopes', () => {
  test('lists scopes in the default scenario', async ({ page }) => {
    await gotoReady(page, '/oauth-scope');

    await expect(
      page.getByRole('heading', { name: 'OAuth / Oidc Scopes' })
    ).toBeVisible();
    await expect(page.getByText('profile').first()).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no scopes', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/oauth-scope');

    await expect(page.getByText('No results.')).toBeVisible();
  });

  test('creates a scope and navigates to its detail page', async ({ page }) => {
    await gotoReady(page, '/oauth-scope/create');

    await page.getByPlaceholder('Enter scope name').fill('Custom Scope');
    await page.getByPlaceholder('Enter scope', { exact: true }).fill('custom');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-scope\/scope-new/);
  });

  test('rejects a scope with no name', async ({ page }) => {
    await gotoReady(page, '/oauth-scope/create');

    await page.getByPlaceholder('Enter scope', { exact: true }).fill('custom');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-scope\/create/);
  });

  test('disables deletion of a default scope', async ({ page }) => {
    await gotoReady(page, '/oauth-scope/scope-1');

    await expect(page.getByRole('heading', { name: /Scope:/ })).toContainText(
      'profile'
    );
    // the mocked scope ("profile") is a built-in default and cannot be removed.
    await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled();
  });
});

test.describe('oauth policies', () => {
  test('lists policies in the default scenario', async ({ page }) => {
    await gotoReady(page, '/oauth-policy');

    await expect(
      page.getByRole('heading', { name: 'OAuth / Oidc Policies' })
    ).toBeVisible();
    await expect(page.getByText('Group Policy').first()).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no policies', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/oauth-policy');

    await expect(page.getByText('No results.')).toBeVisible();
  });

  test('creates a policy and navigates to its detail page', async ({
    page
  }) => {
    await gotoReady(page, '/oauth-policy/create');

    await page.getByPlaceholder('Enter policy name').fill('Role Policy');
    await page.getByPlaceholder('Enter claim name').fill('roles');
    await page.getByPlaceholder('Enter default value').fill('user');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-policy\/policy-new/);
  });

  test('rejects a policy with no name', async ({ page }) => {
    await gotoReady(page, '/oauth-policy/create');

    await page.getByPlaceholder('Enter claim name').fill('roles');
    await page.getByPlaceholder('Enter default value').fill('user');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/oauth-policy\/create/);
  });

  test('renders policy detail and deletes the policy', async ({ page }) => {
    await gotoReady(page, '/oauth-policy/policy-1');

    await expect(page.getByRole('heading', { name: /Policy:/ })).toContainText(
      'Group Policy'
    );

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the policy Group Policy?')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page).toHaveURL(/\/oauth-policy$/);
  });
});
