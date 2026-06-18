import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { gotoReady } from '$test_helpers/layout';

test.describe('app login', () => {
  test('shows the QR code and offers to cancel', async ({ page }) => {
    await gotoReady(page, '/login');

    await page.getByRole('button', { name: 'App Login' }).click();

    // The websocket mock pushes an initial code, which renders the QR image.
    await expect(page.getByAltText('QR Code')).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Open in App' })
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();

    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByAltText('QR Code')).toHaveCount(0);
  });

  test('"Open in App" opens the positron:// deep link with the device code', async ({
    page
  }) => {
    // Custom-protocol navigations are blocked in the browser, so record the
    // Href the anchor would have opened instead of letting it navigate.
    await page.addInitScript(() => {
      const w = window as unknown as { __deepLink?: string };
      // oxlint-disable-next-line unbound-method
      const realClick = HTMLAnchorElement.prototype.click;
      HTMLAnchorElement.prototype.click = function click(
        this: HTMLAnchorElement
      ) {
        if (this.href.startsWith('positron://')) {
          // oxlint-disable-next-line no-underscore-dangle
          w.__deepLink = this.href;
          return;
        }
        realClick.call(this);
      };
    });

    await gotoReady(page, '/login');
    await page.getByRole('button', { name: 'App Login' }).click();
    await expect(page.getByAltText('QR Code')).toBeVisible();

    const openInApp = page.getByRole('button', { name: 'Open in App' });
    await expect(openInApp).toBeEnabled();
    await openInApp.click();

    const deepLink = await page.evaluate(
      // oxlint-disable-next-line no-underscore-dangle
      () => (window as unknown as { __deepLink?: string }).__deepLink
    );
    expect(deepLink).toBeDefined();

    // oxlint-disable-next-line non-nullable-type-assertion-style
    const url = new URL(deepLink as string);
    expect(url.protocol).toBe('positron:');
    expect(url.host).toBe('login');
    expect(url.searchParams.get('code')).toBeTruthy();
    expect(url.searchParams.get('redirect')).toBe(new URL(page.url()).origin);
  });
});
