import { describe, expect, it } from 'vitest';
import {
  HorizontalLayout,
  VerticalLayout,
  getSettings,
  saveSettings
} from '$lib/commands/settings.svelte';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

const settings = {
  height: 500,
  horizontal_layout: HorizontalLayout.Center,
  vertical_layout: VerticalLayout.Center,
  width: 800
};

describe('getSettings', () => {
  it('returns the backend settings object', async () => {
    mockCommand('get_settings', settings);
    expect(await getSettings()).toEqual(settings);
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('get_settings');
    expect(await getSettings()).toBeUndefined();
  });
});

describe('saveSettings', () => {
  it('forwards the settings and returns true on success', async () => {
    let received: unknown = undefined;
    mockCommands((cmd, payload) => {
      if (cmd === 'save_settings') {
        received = payload;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await saveSettings(settings)).toBe(true);
    expect(received).toEqual({ settings });
  });

  it('returns false when the command fails', async () => {
    mockCommandError('save_settings');
    expect(await saveSettings(settings)).toBe(false);
  });
});
