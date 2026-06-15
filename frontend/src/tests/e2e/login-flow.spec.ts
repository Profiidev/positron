import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { gotoReady } from '$test_helpers/layout';

test.describe('app login', () => {
  test('shows the QR code and offers to cancel', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByRole('button', { name: 'App Login' }).click();

    // The websocket mock pushes an initial code, which renders the QR image.
    await expect(page.getByAltText('QR Code')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();

    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByAltText('QR Code')).toHaveCount(0);
  });
});
