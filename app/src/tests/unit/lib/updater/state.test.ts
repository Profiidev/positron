import { describe, expect, it, vi } from 'vitest';
import {
  authStatusState,
  noteUsersState,
  notesConfigState,
  notesState,
  onNotesUpdate,
  onUpdate,
  setupStatusState,
  triggerUpdates,
  userInfoState
} from '$lib/updater/state.svelte';
import { UpdateMessageType } from '$lib/updater/types.svelte';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

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

  it('keeps the prior value (null) when the command fails', async () => {
    // After the "prevent overwriting updates" fix (`value = v ?? value`), a
    // Failing command no longer clobbers the state: userInfoState has never
    // Loaded here, so update() leaves it at its initial null rather than undefined.
    mockCommands((cmd) => {
      if (cmd === 'user_info') {
        throw new Error('boom');
      }
      throw new Error('unexpected');
    });
    await userInfoState.update();
    expect(userInfoState.value).toBeNull();
  });

  it('triggerUpdates is a no-op when nothing is subscribed', () => {
    expect(() => triggerUpdates()).not.toThrow();
  });

  it('keeps the previous value when a refresh returns undefined', async () => {
    // The "prevent overwriting updates" fix: `value = v ?? value`. A failing
    // Command (listNotes swallows errors into undefined) must not wipe state.
    mockCommand('list_notes', [{ id: 'n1' }]);
    await notesState.update();
    expect(notesState.value).toEqual([{ id: 'n1' }]);

    mockCommandError('list_notes');
    await notesState.update();
    expect(notesState.value).toEqual([{ id: 'n1' }]);
  });

  it('notesConfigState populates from notes_config', async () => {
    mockCommand('notes_config', { max_per_user: 3 });
    await notesConfigState.update();
    expect(notesConfigState.value).toEqual({ max_per_user: 3 });
  });

  it('noteUsersState populates from list_users_note', async () => {
    mockCommand('list_users_note', [{ id: 'u1', name: 'Ada' }]);
    await noteUsersState.update();
    expect(noteUsersState.value).toEqual([{ id: 'u1', name: 'Ada' }]);
  });
});

describe('onUpdate / onNotesUpdate', () => {
  it('fires the callback only for its own message type and stops after unsubscribe', () => {
    const cb = vi.fn();
    const unsubscribe = onUpdate(UpdateMessageType.UsersUpdated, cb);

    triggerUpdates(UpdateMessageType.NotesUpdated);
    expect(cb).not.toHaveBeenCalled();

    triggerUpdates(UpdateMessageType.UsersUpdated);
    expect(cb).toHaveBeenCalledTimes(1);

    // An untyped trigger fans out to every registered type.
    triggerUpdates();
    expect(cb).toHaveBeenCalledTimes(2);

    unsubscribe();
    triggerUpdates(UpdateMessageType.UsersUpdated);
    expect(cb).toHaveBeenCalledTimes(2);
  });

  it('onNotesUpdate registers under the NotesUpdated type', () => {
    const cb = vi.fn();
    const unsubscribe = onNotesUpdate(cb);

    triggerUpdates(UpdateMessageType.UsersUpdated);
    expect(cb).not.toHaveBeenCalled();

    triggerUpdates(UpdateMessageType.NotesUpdated);
    expect(cb).toHaveBeenCalledTimes(1);

    unsubscribe();
    triggerUpdates(UpdateMessageType.NotesUpdated);
    expect(cb).toHaveBeenCalledTimes(1);
  });
});
