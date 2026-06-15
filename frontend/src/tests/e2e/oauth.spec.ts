import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
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

  test('saves changes to a client', async ({ page }) => {
    await gotoReady(page, '/oauth-client/client-1');

    const name = page.getByPlaceholder('Enter client name');
    await expect(name).toHaveValue('Dashboard App');
    await name.fill('Dashboard App 2');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('Client Dashboard App updated successfully')
    ).toBeVisible();
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

  test('saves changes to a scope', async ({ page }) => {
    await gotoReady(page, '/oauth-scope/scope-1');

    const name = page.getByPlaceholder('Enter scope name');
    await expect(name).toHaveValue('profile');
    await name.fill('profile renamed');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('Scope profile updated successfully')
    ).toBeVisible();
  });

  test('disables deletion of a default scope', async ({ page }) => {
    await gotoReady(page, '/oauth-scope/scope-1');

    await expect(page.getByRole('heading', { name: /Scope:/ })).toContainText(
      'profile'
    );
    // The mocked scope ("profile") is a built-in default and cannot be removed.
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

  test('saves changes to a policy', async ({ page }) => {
    await gotoReady(page, '/oauth-policy/policy-1');

    const name = page.getByPlaceholder('Enter policy name');
    await expect(name).toHaveValue('Group Policy');
    await name.fill('Group Policy 2');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('Policy Group Policy updated successfully')
    ).toBeVisible();
  });

  test('edits a group-mapping claim value and saves', async ({ page }) => {
    await gotoReady(page, '/oauth-policy/policy-1');

    await expect(page.getByText('Group Mapping')).toBeVisible();
    const claim = page.getByPlaceholder('Claim value');
    await expect(claim).toHaveValue('admin');
    await claim.fill('superadmin');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('Policy Group Policy updated successfully')
    ).toBeVisible();
  });

  test('removes a group mapping and saves', async ({ page }) => {
    await gotoReady(page, '/oauth-policy/policy-1');

    const claim = page.getByPlaceholder('Claim value');
    await expect(claim).toBeVisible();
    // The mapping row holds the move/select/claim/delete controls; the last
    // Button in the row is the destructive remove action.
    await claim.locator('..').getByRole('button').last().click();
    await expect(page.getByPlaceholder('Claim value')).toHaveCount(0);

    await page.getByRole('button', { name: 'Save Changes' }).click();
    await expect(
      page.getByText('Policy Group Policy updated successfully')
    ).toBeVisible();
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
