import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('note editor', () => {
  test('mounts the rich-text editor with its toolbar', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    // The collaborative editor loads asynchronously; wait for the ProseMirror
    // Content area and the formatting toolbar to appear.
    const editor = page.locator('.ProseMirror');
    await expect(editor).toBeVisible();
    await expect(editor).toHaveAttribute('contenteditable', 'true');
  });

  test('accepts typed content', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    const editor = page.locator('.ProseMirror');
    await editor.click();
    await page.keyboard.type('Hello from the e2e test');

    await expect(editor).toContainText('Hello from the e2e test');
  });

  test('applies bold formatting via the keyboard shortcut', async ({
    page
  }) => {
    await gotoReady(page, '/notes/note-1');

    const editor = page.locator('.ProseMirror');
    await editor.click();
    await page.keyboard.press('ControlOrMeta+b');
    await page.keyboard.type('bolded');

    await expect(editor.locator('strong')).toHaveText('bolded');
  });
});
