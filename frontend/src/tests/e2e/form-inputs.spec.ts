import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('multiselect dropdown', () => {
  test('selects and deselects an option', async ({ page }) => {
    await gotoReady(page, '/oauth-client/create');

    // The Scopes field is the only dropdown on the create form; it starts empty.
    // Once open, the popover's search box is also a combobox, so target the
    // Trigger button specifically.
    const dropdown = page.locator('button[role="combobox"]');
    await expect(dropdown).toHaveText(/No Scopes/);

    await dropdown.click();
    await expect(page.getByPlaceholder('Search scopes...')).toBeVisible();

    // Picking an option from the popover updates the trigger label.
    await page.getByRole('option', { name: 'profile' }).click();
    await expect(dropdown).toHaveText(/profile/);

    // A multiselect keeps the popover open, so clicking again toggles it off.
    await page.getByRole('option', { name: 'profile' }).click();
    await expect(dropdown).toHaveText(/No Scopes/);
  });

  test('filters options through the search box', async ({ page }) => {
    await gotoReady(page, '/oauth-client/create');

    await page.locator('button[role="combobox"]').click();
    await page.getByPlaceholder('Search scopes...').fill('does-not-exist');

    await expect(page.getByText('No Scopes found')).toBeVisible();
  });

  test('deselects a pre-selected group on the user detail page', async ({
    page
  }) => {
    await gotoReady(page, '/users/user-1');

    // Bob belongs to the Admins group, so the dropdown shows it as selected.
    const dropdown = page.locator('button[role="combobox"]');
    await expect(dropdown).toHaveText(/Admins/);

    await dropdown.click();
    await page.getByRole('option', { name: 'Admins' }).click();

    await expect(dropdown).toHaveText(/No Group Membership/);
  });
});

test.describe('tags input', () => {
  test('adds a redirect URI tag on Enter', async ({ page }) => {
    await gotoReady(page, '/oauth-client/client-1');

    // The redirect_uri input and the tags input share a placeholder; the tags
    // Input is the second one (Additional Redirect URIs).
    const tagsInput = page
      .getByPlaceholder('https://example.com/callback')
      .nth(1);
    await tagsInput.fill('https://extra.example/callback');
    await tagsInput.press('Enter');

    await expect(
      page.getByText('https://extra.example/callback')
    ).toBeVisible();
  });

  test('rejects an invalid (non-URL) tag', async ({ page }) => {
    await gotoReady(page, '/oauth-client/client-1');

    const tagsInput = page
      .getByPlaceholder('https://example.com/callback')
      .nth(1);
    await tagsInput.fill('not a url');
    await tagsInput.press('Enter');

    // Invalid input is not committed as a tag.
    await expect(page.getByText('not a url')).toHaveCount(0);
  });
});
