import { $, browser, expect } from '@wdio/globals';
import { runIpcCommand } from '../helpers/app-launcher.js';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  authenticateViaDeepLink,
  byButton,
  invokeCommand,
  isMobile,
  reloadApp,
  resetAppData,
  resetMockState,
  seedSetup,
  waitForBodyText,
  waitForRoute
} from '../helpers/test-utils.js';

const WIDTH_INPUT = 'input[name="width"]';
const HEIGHT_INPUT = 'input[name="height"]';

interface AppSettings {
  horizontal_layout: string;
  vertical_layout: string;
  width: number;
  height: number;
}

interface NoteInfo {
  id: string;
}

// Matches `Settings::default()` in `src-tauri/src/store.rs` (800x500, centered).
const DEFAULT_SETTINGS: AppSettings = {
  height: 500,
  horizontal_layout: 'Center',
  vertical_layout: 'Center',
  width: 800
};

/**
 * Covers the desktop-only functionality added on this branch: the layer-shell
 * window `Settings` page (`app/src/routes/(app)/settings`) and the
 * `positron ipc show|hide|toggle|open ...` control CLI
 * (`src-tauri/src/linux/{cli,ipc}.rs`). Neither exists on mobile - the CLI
 * module isn't even compiled there - so this whole suite is desktop/iOS-excluded
 * (skips on Android/iOS via `isMobile()`), unlike the shared specs elsewhere.
 */
describe('Desktop app features', () => {
  // oxlint-disable-next-line func-names
  before(function () {
    if (isMobile()) {
      this.skip();
    }
  });

  beforeEach(async () => {
    await resetMockState();
    await resetAppData();
    await seedSetup(MOCK_URL);
    await authenticateViaDeepLink();
    await waitForRoute('/');

    // Settings live in a real on-disk store that `resetAppData` doesn't touch
    // (it only clears auth/setup state), so a value saved by one test would
    // otherwise leak into every test that runs after it. Pin a known baseline
    // here instead of assuming defaults are still in effect.
    await invokeCommand('save_settings', { settings: DEFAULT_SETTINGS });
  });

  describe('Settings page', () => {
    it('navigates to /settings from the nav', async () => {
      await byButton('Settings').click();
      await waitForRoute('/settings');
    });

    it('pre-fills the form with the persisted defaults', async () => {
      await byButton('Settings').click();
      await waitForRoute('/settings');

      // Defaults from `Settings::default()` (src-tauri/src/store.rs): 800x500.
      await expect($(WIDTH_INPUT)).toHaveValue('800');
      await expect($(HEIGHT_INPUT)).toHaveValue('500');
    });

    it('saves changed settings and persists them across a reload', async () => {
      await byButton('Settings').click();
      await waitForRoute('/settings');

      await $(WIDTH_INPUT).setValue('1024');
      await $(HEIGHT_INPUT).setValue('700');
      await byButton('Save').click();

      await waitForBodyText('saved successfully');

      const saved = await invokeCommand<AppSettings>('get_settings');
      expect(saved.width).toBe(1024);
      expect(saved.height).toBe(700);

      // Reload re-fetches settings from the Rust store, not just local form
      // state, so this proves the value actually persisted server-side.
      await reloadApp();
      await waitForRoute('/settings');
      await expect($(WIDTH_INPUT)).toHaveValue('1024');
      await expect($(HEIGHT_INPUT)).toHaveValue('700');
    });

    it('rejects a width below the minimum without saving', async () => {
      await byButton('Settings').click();
      await waitForRoute('/settings');

      await $(WIDTH_INPUT).setValue('100');
      await byButton('Save').click();

      // Client-side zod validation (`min(500)`) blocks the submit handler, so
      // the app never calls `save_settings` and the stored value is untouched.
      await browser.pause(800);
      const settings = await invokeCommand<AppSettings>('get_settings');
      expect(settings.width).toBe(800);
    });
  });

  describe('IPC control CLI', () => {
    it('opens settings via the ipc CLI', () => {
      const result = runIpcCommand(['open', 'settings']);
      expect(result.success).toBe(true);
    });

    it('opens settings via the ipc CLI, waiting for the route', async () => {
      const result = runIpcCommand(['open', 'settings']);
      expect(result.success).toBe(true);
      await waitForRoute('/settings');
    });

    it('opens notes via the ipc CLI', async () => {
      await byButton('Settings').click();
      await waitForRoute('/settings');

      const result = runIpcCommand(['open', 'notes']);
      expect(result.success).toBe(true);
      await waitForRoute('/');
    });

    it('opens a specific note via the ipc CLI', async () => {
      const note = await invokeCommand<NoteInfo>('create_note', {
        title: 'IPC Note'
      });

      const result = runIpcCommand(['open', 'note', note.id]);
      expect(result.success).toBe(true);
      await waitForRoute(`/notes/${note.id}`);
    });

    it('shows, hides and toggles the window without erroring', () => {
      expect(runIpcCommand(['show']).success).toBe(true);
      expect(runIpcCommand(['hide']).success).toBe(true);
      expect(runIpcCommand(['toggle']).success).toBe(true);
      expect(runIpcCommand(['toggle']).success).toBe(true);
    });
  });
});
