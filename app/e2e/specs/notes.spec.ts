import { $, browser, expect } from '@wdio/globals';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  authenticateViaDeepLink,
  byButton,
  getRoute,
  invokeCommand,
  reloadApp,
  resetAppData,
  resetMockState,
  seedSetup,
  setNoteLimit,
  waitForBodyText,
  waitForElementVisible,
  waitForRoute
} from '../helpers/test-utils.js';

const CREATE_LINK = 'a[href="/notes/create"]';
const TITLE_INPUT = 'input[placeholder="Enter note title"]';
// The trash button lives inside a note card link (`/notes/<uuid>`); the Create
// link (`/notes/create`) has no child button, so this only matches a real card.
const CARD_TRASH = 'a[href^="/notes/"] button';

/**
 * Covers the app notes feature added on this branch: the notes list (the main
 * page), creating notes through the form, deleting them, and the per-user quota.
 * The notes commands talk to the backend, so the mock server's stateful
 * `/api/notes/management` endpoints stand in for it. Live collaborative editing
 * (the `/notes/<id>` Yjs editor) needs a real note websocket the headless mock
 * can't serve, so it's only exercised as far as create/open navigation.
 */
describe('Notes', () => {
  // Authentication is delivered over adb deep links, so Android only.
  // oxlint-disable-next-line func-names
  before(function () {
    if (process.env.TAURI_TEST_PLATFORM !== 'android') {
      this.skip();
    }
  });

  beforeEach(async () => {
    await resetMockState();
    await resetAppData();
    await seedSetup(MOCK_URL);
    await authenticateViaDeepLink();
    await waitForRoute('/');
  });

  it('shows an empty state with a Create action', async () => {
    await waitForBodyText('No notes yet');
    await expect($(CREATE_LINK)).toBeDisplayed();
  });

  it('lists notes returned by the server', async () => {
    await invokeCommand('create_note', { title: 'Alpha Note' });
    await invokeCommand('create_note', { title: 'Beta Note' });

    // The list only refetches on (re)mount, so reload to pick the seeds up.
    await reloadApp();
    await waitForRoute('/');

    await waitForBodyText('Alpha Note');
    await waitForBodyText('Beta Note');
  });

  it('creates a note through the form and shows it in the list', async () => {
    const title = `E2E Note ${Date.now()}`;

    await $(CREATE_LINK).click();
    await waitForRoute('/notes/create');

    await $(TITLE_INPUT).setValue(title);
    await byButton('Create').click();

    // A successful create routes to the new note's editor (`/notes/<uuid>`).
    await browser.waitUntil(
      async () => {
        const route = await getRoute();
        return route.startsWith('/notes/') && route !== '/notes/create';
      },
      { timeout: 15_000, timeoutMsg: 'did not navigate to the created note' }
    );

    // Back to the list via the editor's back arrow; the note must be there.
    await $('a[href="/"]').click();
    await waitForRoute('/');
    await waitForBodyText(title);
  });

  it('requires a title before it can create', async () => {
    await $(CREATE_LINK).click();
    await waitForRoute('/notes/create');

    // Empty title fails the schema, so the submit handler never runs.
    await byButton('Create').click();
    await browser.pause(800);
    expect(await getRoute()).toBe('/notes/create');
  });

  it('deletes a note from the list', async () => {
    await invokeCommand('create_note', { title: 'Deletable Note' });
    await reloadApp();
    await waitForRoute('/');
    await waitForBodyText('Deletable Note');

    const trash = await waitForElementVisible(CARD_TRASH);
    await trash.click();
    await byButton('Delete').click();

    // Without the updater websocket the list can't refresh itself; reload to
    // confirm the note is gone from the server.
    await reloadApp();
    await waitForRoute('/');
    await waitForBodyText('No notes yet');
  });

  it('disables the Create action at the note limit', async () => {
    await invokeCommand('create_note', { title: 'Only Note' });
    await setNoteLimit(1);

    await reloadApp();
    await waitForRoute('/');
    await waitForBodyText('Only Note');

    // At the quota the Create action drops its href and renders disabled.
    await expect($(CREATE_LINK)).not.toBeExisting();
    await expect(byButton('Create')).toBeDisabled();
  });
});
