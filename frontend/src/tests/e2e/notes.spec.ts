import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e-fixture';
import { setupSession } from '$test_helpers/session';
import { expectNoHorizontalOverflow, gotoReady } from '$test_helpers/layout';

test.beforeEach(async ({ context }) => setupSession(context));

test.describe('notes list', () => {
  test('lists notes in the default scenario', async ({ page }) => {
    await gotoReady(page, '/notes');

    await expect(page.getByRole('heading', { name: 'Notes' })).toBeVisible();
    await expect(page.getByText('My First Note')).toBeVisible();
    await expect(page.getByText('First note preview')).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('shows an empty state with no notes', async ({ context, page }) => {
    await setupSession(context, 'empty');
    await gotoReady(page, '/notes');

    await expect(page.getByText('No notes yet')).toBeVisible();
  });

  test('disables create when the note limit is reached', async ({
    context,
    page
  }) => {
    await setupSession(context, 'at-limit');
    await gotoReady(page, '/notes');

    await expect(page.getByRole('button', { name: 'Create' })).toBeDisabled();
  });
});

test.describe('note create', () => {
  test('creates a note and leaves the create page', async ({ page }) => {
    await gotoReady(page, '/notes/create');

    await page.getByPlaceholder('Enter note title').fill('Meeting notes');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/notes\/(?!create)/);
  });

  test('rejects an empty title', async ({ page }) => {
    await gotoReady(page, '/notes/create');

    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/notes\/create/);
  });

  test('navigates to the new note when creating the last allowed note', async ({
    page
  }) => {
    await page.route('**/api/notes/management**', async (route) => {
      const url = route.request().url();
      const method = route.request().method();

      // Config caps the user at a single note so the create page allows it.
      if (method === 'GET' && url.endsWith('/config')) {
        await route.fulfill({ json: { max_per_user: 1 } });
        return;
      }

      // Creating returns the id the page then navigates to.
      if (method === 'POST') {
        await route.fulfill({ json: { id: 'note-new' } });
        return;
      }

      // The detail page loads the freshly created note.
      if (method === 'GET' && url.endsWith('/note-new')) {
        await route.fulfill({
          json: {
            can_edit: true,
            id: 'note-new',
            is_owner: true,
            owner: { id: 'user-admin', name: 'Ada Admin' },
            shared_with: [],
            title: 'Final note'
          }
        });
        return;
      }

      // The notes list starts empty so the limit is not yet reached.
      if (method === 'GET' && url.endsWith('/management')) {
        await route.fulfill({ json: [] });
        return;
      }

      // Everything else (e.g. the share user list) falls through to MSW.
      await route.fallback();
    });

    await gotoReady(page, '/notes/create');

    await page.getByPlaceholder('Enter note title').fill('Final note');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(page).toHaveURL(/\/notes\/note-new$/);
    await expect(page.getByPlaceholder('Note title')).toHaveValue('Final note');
  });

  test('shows a limit error when create returns conflict', async ({ page }) => {
    await page.route('**/api/notes/management', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({ body: '{}', status: 409 });
        return;
      }
      await route.continue();
    });

    await gotoReady(page, '/notes/create');

    await page.getByPlaceholder('Enter note title').fill('Another note');
    await page.getByRole('button', { name: 'Create' }).click();

    await expect(
      page.getByText('You have reached the maximum number of notes.')
    ).toBeVisible();
    await expect(page).toHaveURL(/\/notes\/create/);
  });
});

test.describe('note detail', () => {
  test('renders the editor with the note title', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await expect(page.getByPlaceholder('Note title')).toHaveValue(
      'My First Note'
    );
    await expectNoHorizontalOverflow(page);
  });

  test('saves a renamed title on blur', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    const title = page.getByPlaceholder('Note title');
    await expect(title).toHaveValue('My First Note');
    await title.fill('Renamed Note');
    // Pressing Enter blurs the input, which triggers the title save.
    await title.press('Enter');

    await expect(page.getByText('Title updated')).toBeVisible();
  });

  test('shares the note with another user', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Share' }).click();
    await expect(page.getByPlaceholder('Search people...')).toBeVisible();
    await page
      .getByRole('option', { name: 'Cara User' })
      .getByRole('button', { name: 'Edit' })
      .click();

    // The share update is debounced (~500ms) before it persists.
    await expect(page.getByText('Shared users updated')).toBeVisible();
  });

  test('shares the note as view-only', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Share' }).click();
    await expect(page.getByPlaceholder('Search people...')).toBeVisible();
    await page
      .getByRole('option', { name: 'Cara User' })
      .getByRole('button', { name: 'View' })
      .click();

    await expect(page.getByText('Shared users updated')).toBeVisible();
  });

  test('locks the editor for a view-only note', async ({ context, page }) => {
    await setupSession(context, 'readonly');
    await gotoReady(page, '/notes/note-1');

    // A non-owner viewer cannot rename, delete or re-share the note.
    await expect(page.getByPlaceholder('Note title')).toHaveAttribute(
      'readonly'
    );
    await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Share' })).toHaveCount(0);
    await expect(
      page.getByRole('button', { name: /Transfer ownership/ })
    ).toHaveCount(0);

    // The editor renders read-only with no contenteditable surface.
    const editor = page.locator('.ProseMirror').first();
    await expect(editor).toHaveAttribute('contenteditable', 'false');
  });

  test('transfers ownership through the confirmation dialog', async ({
    page
  }) => {
    let transferRequest: { note_id: string; new_owner_id: string } | undefined =
      undefined;
    await page.route('**/api/notes/management/transfer', async (route) => {
      transferRequest = route.request().postDataJSON();
      await route.fulfill({ body: '{}', status: 200 });
    });

    await gotoReady(page, '/notes/note-1');

    await page
      .getByRole('button', { name: 'Transfer ownership from Bob User' })
      .click();
    await page.getByRole('option', { name: 'Cara User' }).click();
    await expect(
      page.getByText(
        'Transfer ownership of "My First Note" to Cara User? You will remain an editor but lose owner controls.'
      )
    ).toBeVisible();
    await page.getByRole('button', { name: 'Transfer' }).last().click();

    /*
     * Owner controls only flip once the backend pushes a `Note` update over the
     * websocket (a no-op in e2e), so assert the success toast plus that the
     * transfer request carried the chosen new owner.
     */
    await expect(page.getByText('Ownership transferred')).toBeVisible();
    expect(transferRequest).toEqual({
      new_owner_id: 'user-2',
      note_id: 'note-1'
    });
  });

  test('shows a limit error when transfer returns conflict', async ({
    context,
    page
  }) => {
    await setupSession(context, 'transfer-at-limit');
    await gotoReady(page, '/notes/note-1');

    await page
      .getByRole('button', { name: 'Transfer ownership from Bob User' })
      .click();
    await page.getByRole('option', { name: 'Cara User' }).click();
    await page.getByRole('button', { name: 'Transfer' }).last().click();

    await expect(
      page.getByText('Cara User has reached the maximum number of notes.')
    ).toBeVisible();
  });

  test('deletes a note through the confirmation dialog', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the note My First Note?')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page).toHaveURL(/\/notes$/);
  });
});

test.describe('note snapshots', () => {
  test('shows the snapshot manager only for the owner', async ({
    context,
    page
  }) => {
    await gotoReady(page, '/notes/note-1');
    await expect(
      page.getByRole('button', { name: 'Snapshot manager' })
    ).toBeVisible();

    // A view-only viewer is not the owner, so the manager is hidden.
    await setupSession(context, 'readonly');
    await gotoReady(page, '/notes/note-1');
    await expect(
      page.getByRole('button', { name: 'Snapshot manager' })
    ).toHaveCount(0);
  });

  test('opens a snapshot from the manager', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Snapshot manager' }).click();
    await expect(page.getByPlaceholder('Search new owner...')).toBeVisible();
    await page.getByRole('option').first().click();

    await expect(page).toHaveURL(/\/notes\/note-1\/snapshot-1$/);
  });

  test('restores a snapshot from the manager', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Snapshot manager' }).click();
    // The restore button lives inside the snapshot option (ArchiveRestore icon).
    await page
      .getByRole('option')
      .first()
      .getByRole('button')
      .first()
      .click();

    await expect(
      page.getByText('Do you really want to restore this snapshot')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Restore' }).last().click();

    await expect(page.getByText('Snapshot restored successfully')).toBeVisible();
  });

  test('deletes a snapshot from the manager', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Snapshot manager' }).click();
    // The destructive delete button is the last button inside the option.
    await page
      .getByRole('option')
      .first()
      .getByRole('button')
      .last()
      .click();

    await expect(
      page.getByText('Do you really want to delete the snapshot')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page.getByText('Snapshot deleted successfully')).toBeVisible();
  });

  test('renders the snapshot view page with restore and delete', async ({
    page
  }) => {
    await gotoReady(page, '/notes/note-1/snapshot-1');

    await expect(page.getByText('My First Note:')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Restore' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Delete' })).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });

  test('restores from the snapshot view page', async ({ page }) => {
    await gotoReady(page, '/notes/note-1/snapshot-1');

    await page.getByRole('button', { name: 'Restore' }).click();
    await expect(
      page.getByText('Do you really want to restore this snapshot')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Restore' }).last().click();

    await expect(page.getByText('Snapshot restored successfully')).toBeVisible();
    await expect(page).toHaveURL(/\/notes\/note-1$/);
  });

  test('deletes from the snapshot view page', async ({ page }) => {
    await gotoReady(page, '/notes/note-1/snapshot-1');

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the snapshot')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page.getByText('Snapshot deleted successfully')).toBeVisible();
    await expect(page).toHaveURL(/\/notes\/note-1$/);
  });

  test('redirects to the note when the snapshot is missing', async ({
    page
  }) => {
    await gotoReady(page, '/notes/note-1/snapshot-missing');

    // The view page bounces back to the note with an error flag, which the note
    // Page surfaces as a toast and then strips from the URL.
    await expect(page).toHaveURL(/\/notes\/note-1$/);
    await expect(page.getByText('Snapshot not found')).toBeVisible();
  });
});
