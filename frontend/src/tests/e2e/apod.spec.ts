import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test('redirects /apod to today', async ({ page }) => {
  await gotoReady(page, '/apod');
  await expect(page).toHaveURL(/\/apod\/\d{4}-\d{2}-\d{2}/);
});

test.describe('apod today', () => {
  test('shows the selected image of the day', async ({ page }) => {
    await gotoReady(page, '/apod');

    await expect(page.getByRole('tab', { name: 'Today' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Library' })).toBeVisible();
    await expect(page.getByText('Spiral Galaxy')).toBeVisible();
    // The mocked image is already chosen, so the control offers to deselect.
    await expect(page.getByRole('button', { name: 'Deselect' })).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('switches to the library tab', async ({ page }) => {
    await gotoReady(page, '/apod');
    await page.getByRole('tab', { name: 'Library' }).click();

    await expect(page).toHaveURL(/\/apod\/list/);
  });
});

test.describe('apod library', () => {
  test('lists the selected images', async ({ page, context }) => {
    await setupSession(context);
    await gotoReady(page, '/apod/list');

    await expect(page.getByText('Spiral Galaxy')).toBeVisible();
    await expect(page.getByText('Nebula')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no images', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/apod/list');

    await expect(page.getByText('No APODs selected')).toBeVisible();
  });
});
