import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('users list', () => {
  test('lists users in the default scenario', async ({ page }) => {
    await gotoReady(page, '/users');

    await expect(page.getByRole('heading', { name: 'Users' })).toBeVisible();
    await expect(page.getByText('bob@example.com')).toBeVisible();
    await expect(page.getByText('cara@example.com')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no users', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/users');

    await expect(page.getByText('No results.')).toBeVisible();
  });
});

test.describe('user create', () => {
  test('creates a user and navigates to its detail page', async ({ page }) => {
    await gotoReady(page, '/users/create');

    await page.getByPlaceholder('Enter user name').fill('New Person');
    await page.getByPlaceholder('Enter email').fill('new.person@example.com');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/users\/user-new/);
  });

  test('rejects an invalid email', async ({ page }) => {
    await gotoReady(page, '/users/create');

    await page.getByPlaceholder('Enter user name').fill('New Person');
    await page.getByPlaceholder('Enter email').fill('not-an-email');
    await page.getByRole('button', { name: 'Create' }).click();

    // Invalid input is rejected, so we stay on the create page.
    await expect(page).toHaveURL(/\/users\/create/);
  });
});

test.describe('user detail', () => {
  test('renders the settings for a user', async ({ page }) => {
    await gotoReady(page, '/users/user-1');

    await expect(page.getByRole('heading', { name: /User:/ })).toContainText(
      'Bob User'
    );
    await expect(page.getByPlaceholder('Enter user name')).toHaveValue(
      'Bob User'
    );
    await expect(page.getByPlaceholder('mail@example.com')).toHaveValue(
      'bob@example.com'
    );
    await expectNoHorizontalOverflow(page);
  });

  test('saves changes to a user', async ({ page }) => {
    await gotoReady(page, '/users/user-1');

    const name = page.getByPlaceholder('Enter user name');
    await expect(name).toHaveValue('Bob User');
    await name.fill('Bob Updated');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(
      page.getByText('User Bob User updated successfully')
    ).toBeVisible();
  });

  test('deletes a user through the confirmation dialog', async ({ page }) => {
    await gotoReady(page, '/users/user-1');

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the user Bob User?')
    ).toBeVisible();

    // The dialog confirm button is the second "Delete" button on the page.
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page).toHaveURL(/\/users$/);
  });
});
