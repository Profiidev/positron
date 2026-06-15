import { expect } from '@playwright/test';
import { test } from '$test_helpers/e2e_fixture';
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
});

test.describe('note detail', () => {
  test('renders the editor with the note title', async ({ page }) => {
    await gotoReady(page, '/notes/note-1');

    await expect(page.getByPlaceholder('Note title')).toHaveValue(
      'My First Note'
    );
    await expectNoHorizontalOverflow(page);
  });

  test('deletes a note through the confirmation dialog', async ({
    page
  }, testInfo) => {
    test.skip(
      (testInfo.project.use.viewport?.width ?? 0) < 1024,
      'the delete label is icon-only on small viewports'
    );

    await gotoReady(page, '/notes/note-1');

    await page.getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('Do you really want to delete the note My First Note?')
    ).toBeVisible();
    await page.getByRole('button', { name: 'Delete' }).last().click();

    await expect(page).toHaveURL(/\/notes$/);
  });
});
