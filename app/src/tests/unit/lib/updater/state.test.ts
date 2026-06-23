import { describe, expect, it } from 'vitest';
import {
  authStatusState,
  setupStatusState,
  triggerUpdates,
  userInfoState
} from '$lib/updater/state.svelte';
import { mockCommand, mockCommands } from '$test_helpers/tauri';

describe('updater state', () => {
  it('starts at null before any update', () => {
    // The state begins life as null (distinct from undefined, which means "the
    // Command ran but the backend returned nothing").
    expect(authStatusState.value).toBeNull();
  });

  it('update() populates value from the backing command', async () => {
    mockCommand('setup_status', { url: 'https://example.com' });
    await setupStatusState.update();
    expect(setupStatusState.value).toEqual({ url: 'https://example.com' });
  });

  it('update() stores undefined when the command fails', async () => {
    mockCommands((cmd) => {
      if (cmd === 'user_info') {
        throw new Error('boom');
      }
      throw new Error('unexpected');
    });
    await userInfoState.update();
    expect(userInfoState.value).toBeUndefined();
  });

  it('triggerUpdates is a no-op when nothing is subscribed', () => {
    expect(() => triggerUpdates()).not.toThrow();
  });
});
