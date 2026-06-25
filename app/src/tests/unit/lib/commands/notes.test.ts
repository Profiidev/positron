import { describe, expect, it } from 'vitest';
import {
  createNote,
  deleteNote,
  deleteNoteSnapshot,
  editNote,
  listNoteSnapshots,
  listNotes,
  listNotesStore,
  listUsersNote,
  noteContent,
  noteInfo,
  noteInfoStore,
  noteSnapshotContent,
  noteSnapshotInfo,
  notesConfig,
  restoreNoteSnapshot,
  saveNoteContent,
  shareNote,
  shareNotePublic,
  transferNote
} from '$lib/commands/notes.svelte';
import type { NoteInfo, ShareEntry } from '$lib/commands/notes-types';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

const noteFixture: NoteInfo = {
  can_edit: true,
  id: 'n1',
  is_owner: true,
  owner: { id: 'u1', name: 'Ada' },
  preview: 'hello',
  public_access: null,
  shared_with: [],
  title: 'Note'
};

describe('list/info getters', () => {
  it('listNotesStore returns the stored notes', async () => {
    mockCommand('list_notes_store', [noteFixture]);
    expect(await listNotesStore()).toEqual([noteFixture]);
  });

  it('listNotes returns the notes', async () => {
    mockCommand('list_notes', [noteFixture]);
    expect(await listNotes()).toEqual([noteFixture]);
  });

  it('noteInfoStore forwards the id and returns the note', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'get_note_store') {
        payload = p;
        return noteFixture;
      }
      throw new Error('unexpected');
    });
    expect(await noteInfoStore('n1')).toEqual(noteFixture);
    expect(payload).toEqual({ id: 'n1' });
  });

  it('noteInfo forwards the uuid and returns the note', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'note_info') {
        payload = p;
        return noteFixture;
      }
      throw new Error('unexpected');
    });
    expect(await noteInfo('n1')).toEqual(noteFixture);
    expect(payload).toEqual({ uuid: 'n1' });
  });

  it('notesConfig returns the config', async () => {
    mockCommand('notes_config', { max_per_user: 5 });
    expect(await notesConfig()).toEqual({ max_per_user: 5 });
  });

  it('noteSnapshotInfo forwards the snapshot id', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'note_snapshot_info') {
        payload = p;
        return { created_at: 't', note_id: 'n1', title: 'T' };
      }
      throw new Error('unexpected');
    });
    expect(await noteSnapshotInfo('s1')).toEqual({
      created_at: 't',
      note_id: 'n1',
      title: 'T'
    });
    expect(payload).toEqual({ snapshotId: 's1' });
  });

  it.each([
    ['listNotesStore', listNotesStore, 'list_notes_store'],
    ['listNotes', listNotes, 'list_notes'],
    ['notesConfig', notesConfig, 'notes_config']
  ])('%s returns undefined when the command fails', async (_l, fn, cmd) => {
    mockCommandError(cmd);
    expect(await fn()).toBeUndefined();
  });

  it('noteInfoStore returns undefined on error', async () => {
    mockCommandError('get_note_store');
    expect(await noteInfoStore('n1')).toBeUndefined();
  });

  it('noteInfo returns undefined on error', async () => {
    mockCommandError('note_info');
    expect(await noteInfo('n1')).toBeUndefined();
  });

  it('noteSnapshotInfo returns undefined on error', async () => {
    mockCommandError('note_snapshot_info');
    expect(await noteSnapshotInfo('s1')).toBeUndefined();
  });
});

describe('list getters that default to an empty array', () => {
  it('listUsersNote returns the users', async () => {
    mockCommand('list_users_note', [{ id: 'u1', name: 'Ada' }]);
    expect(await listUsersNote()).toEqual([{ id: 'u1', name: 'Ada' }]);
  });

  it('listUsersNote returns [] on error', async () => {
    mockCommandError('list_users_note');
    expect(await listUsersNote()).toEqual([]);
  });

  it('listNoteSnapshots forwards the note uuid and returns snapshots', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'list_note_snapshots') {
        payload = p;
        return [{ created_at: 't', id: 's1', note_id: 'n1', preview: 'p' }];
      }
      throw new Error('unexpected');
    });
    expect(await listNoteSnapshots('n1')).toHaveLength(1);
    expect(payload).toEqual({ noteUuid: 'n1' });
  });

  it('listNoteSnapshots returns [] on error', async () => {
    mockCommandError('list_note_snapshots');
    expect(await listNoteSnapshots('n1')).toEqual([]);
  });
});

describe('binary content getters', () => {
  it('noteContent forwards the id and wraps bytes in a Uint8Array', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'note_content') {
        payload = p;
        return [1, 2, 3];
      }
      throw new Error('unexpected');
    });
    const res = await noteContent('n1');
    expect(res).toBeInstanceOf(Uint8Array);
    expect([...res!]).toEqual([1, 2, 3]);
    expect(payload).toEqual({ id: 'n1' });
  });

  it('noteSnapshotContent forwards the snapshot id and wraps bytes', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'note_snapshot_content') {
        payload = p;
        return [9, 8];
      }
      throw new Error('unexpected');
    });
    const res = await noteSnapshotContent('s1');
    expect([...res!]).toEqual([9, 8]);
    expect(payload).toEqual({ snapshotId: 's1' });
  });

  it('noteContent returns undefined on error', async () => {
    mockCommandError('note_content');
    expect(await noteContent('n1')).toBeUndefined();
  });

  it('noteSnapshotContent returns undefined on error', async () => {
    mockCommandError('note_snapshot_content');
    expect(await noteSnapshotContent('s1')).toBeUndefined();
  });
});

describe('mutations returning a boolean', () => {
  it('saveNoteContent forwards the content as a plain array and id', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'save_note_content') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await saveNoteContent('n1', new Uint8Array([4, 5]))).toBe(true);
    expect(payload).toEqual({ content: [4, 5], id: 'n1' });
  });

  it('editNote forwards the title', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'edit_note') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await editNote('n1', 'New title')).toBe(true);
    expect(payload).toEqual({ noteId: 'n1', title: 'New title' });
  });

  it('shareNote forwards the share entries', async () => {
    const sharedWith: ShareEntry[] = [{ access: 'edit', user_id: 'u2' }];
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'share_note') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await shareNote('n1', sharedWith)).toBe(true);
    expect(payload).toEqual({ noteId: 'n1', sharedWith });
  });

  it.each([
    ['public view', 'view'],
    ['public edit', 'edit'],
    ['revoked (null)', null]
  ] as const)('shareNotePublic forwards %s access', async (_l, access) => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'share_note_public') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await shareNotePublic('n1', access)).toBe(true);
    expect(payload).toEqual({ noteId: 'n1', publicAccess: access });
  });

  it('restoreNoteSnapshot forwards the snapshot id', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'restore_note_snapshot') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await restoreNoteSnapshot('s1')).toBe(true);
    expect(payload).toEqual({ snapshotId: 's1' });
  });

  it('deleteNote forwards the note id', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'delete_note') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await deleteNote('n1')).toBe(true);
    expect(payload).toEqual({ noteId: 'n1' });
  });

  it('deleteNoteSnapshot forwards the snapshot id', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'delete_note_snapshot') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await deleteNoteSnapshot('s1')).toBe(true);
    expect(payload).toEqual({ snapshotId: 's1' });
  });

  it.each([
    [
      'saveNoteContent',
      async () => saveNoteContent('n1', new Uint8Array()),
      'save_note_content'
    ],
    ['editNote', async () => editNote('n1', 't'), 'edit_note'],
    ['shareNote', async () => shareNote('n1', []), 'share_note'],
    [
      'shareNotePublic',
      async () => shareNotePublic('n1', 'view'),
      'share_note_public'
    ],
    [
      'restoreNoteSnapshot',
      async () => restoreNoteSnapshot('s1'),
      'restore_note_snapshot'
    ],
    ['deleteNote', async () => deleteNote('n1'), 'delete_note'],
    [
      'deleteNoteSnapshot',
      async () => deleteNoteSnapshot('s1'),
      'delete_note_snapshot'
    ]
  ])('%s returns false when the command fails', async (_l, fn, cmd) => {
    mockCommandError(cmd);
    expect(await fn()).toBe(false);
  });
});

describe('createNote', () => {
  it('returns the new id on success', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'create_note') {
        payload = p;
        return { id: 'new-id' };
      }
      throw new Error('unexpected');
    });
    expect(await createNote('Title')).toEqual({ id: 'new-id', ok: true });
    expect(payload).toEqual({ title: 'Title' });
  });

  it('maps a "limit" rejection to the limit error', async () => {
    mockCommands((cmd) => {
      if (cmd === 'create_note') {
        // oxlint-disable-next-line no-throw-literal
        throw 'limit';
      }
      throw new Error('unexpected');
    });
    expect(await createNote('Title')).toEqual({ error: 'limit', ok: false });
  });

  it('maps any other rejection to the other error', async () => {
    mockCommandError('create_note');
    expect(await createNote('Title')).toEqual({ error: 'other', ok: false });
  });
});

describe('transferNote', () => {
  it('forwards the owner and note ids on success', async () => {
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'transfer_note') {
        payload = p;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await transferNote('n1', 'u2')).toEqual({ ok: true });
    expect(payload).toEqual({ newOwnerId: 'u2', noteId: 'n1' });
  });

  it('maps a "limit" rejection to the limit error', async () => {
    mockCommands((cmd) => {
      if (cmd === 'transfer_note') {
        // oxlint-disable-next-line no-throw-literal
        throw 'limit';
      }
      throw new Error('unexpected');
    });
    expect(await transferNote('n1', 'u2')).toEqual({
      error: 'limit',
      ok: false
    });
  });

  it('maps any other rejection to the other error', async () => {
    mockCommandError('transfer_note');
    expect(await transferNote('n1', 'u2')).toEqual({
      error: 'other',
      ok: false
    });
  });
});
