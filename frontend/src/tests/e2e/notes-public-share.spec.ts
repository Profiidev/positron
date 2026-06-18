import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { seedPublicShareVisitor, setupSession } from '$test_helpers/session';
import { gotoReady } from '$test_helpers/layout';

test.describe('public share page (anonymous visitor)', () => {
  test('renders a view-only public note for an anonymous visitor', async ({
    context,
    page
  }) => {
    await seedPublicShareVisitor(context, 'view');
    await gotoReady(page, '/notes/share/note-1');

    // Stays on the share page rather than redirecting to /login or /notes/:id
    await expect(page).toHaveURL(/\/notes\/share\/note-1/);

    // The read-only title and owner attribution render
    await expect(page.getByPlaceholder('Note title')).toHaveValue(
      'Shared Public Note'
    );
    await expect(page.getByPlaceholder('Note title')).toHaveAttribute(
      'readonly'
    );
    await expect(page.getByText('Cara User')).toBeVisible();

    // A view-only share mounts the editor as non-editable
    const editor = page.locator('.ProseMirror');
    await expect(editor).toBeVisible();
    await expect(editor).toHaveAttribute('contenteditable', 'false');
  });

  test('mounts an editable editor when the public share grants edit', async ({
    context,
    page
  }) => {
    await seedPublicShareVisitor(context, 'edit');
    await gotoReady(page, '/notes/share/note-1');

    const editor = page.locator('.ProseMirror');
    await expect(editor).toBeVisible();
    await expect(editor).toHaveAttribute('contenteditable', 'true');
  });

  test('redirects to /login when the public note fails to load', async ({
    context,
    page
  }) => {
    await seedPublicShareVisitor(context, 'view');
    // The public note info endpoint fails -> the page toasts and bounces to /login
    await page.route(
      '**/api/notes/management/note-1/public',
      async (route) => {
        await route.fulfill({ body: '{}', status: 404 });
      }
    );

    await page.goto('/notes/share/note-1');

    await expect(page).toHaveURL(/\/login/);
    await expect(page.getByText('Failed to load note')).toBeVisible();
  });
});

test.describe('public share page (authenticated user)', () => {
  test('redirects a logged-in user to the authenticated note view', async ({
    context,
    page
  }) => {
    // A normal session (has a jwt + non-anonymous `info`) is not an anonymous
    // Visitor, so the share page bounces it to the real note route.
    await setupSession(context);
    await page.goto('/notes/share/note-1');

    await expect(page).toHaveURL(/\/notes\/note-1$/);
  });
});

test.describe('public access control (owner)', () => {
  test('owner enables public access and gets a copy link', async ({ page }) => {
    await page.context().clearCookies();
    await setupSession(page.context());
    await gotoReady(page, '/notes/note-1');

    // Open the share popover and turn on public View access
    await page.getByRole('button', { name: 'Share' }).click();
    await page
      .getByRole('group', { name: 'Public share access' })
      .getByRole('button', { name: 'View' })
      .click();

    await expect(
      page.getByRole('button', { name: 'Copy share link' })
    ).toBeVisible();
  });

  test('rolls back and toasts when enabling public access fails', async ({
    page
  }) => {
    await page.context().clearCookies();
    await setupSession(page.context());
    // The public-share write fails -> the optimistic state must roll back
    await page.route(
      '**/api/notes/management/share/public',
      async (route) => {
        if (route.request().method() === 'PUT') {
          await route.fulfill({ body: '{}', status: 500 });
          return;
        }
        await route.continue();
      }
    );

    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Share' }).click();
    await page
      .getByRole('group', { name: 'Public share access' })
      .getByRole('button', { name: 'View' })
      .click();

    // The failed write surfaces an error toast and reverts the optimistic
    // Public state, so the copy-share-link affordance disappears again
    await expect(
      page.getByText('Failed to update public access')
    ).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Copy share link' })
    ).toBeHidden();
  });
});
