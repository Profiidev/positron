import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/notes/+page';
import { load as createLoad } from '$routes/notes/create/+page';
import { load as detailLoad } from '$routes/notes/[id]/+page';
import { load as snapshotLoad } from '$routes/notes/[id]/[snapshot]/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

const urlOf = (input: RequestInfo | URL): string => {
  if (typeof input === 'string') {
    return input;
  }
  if (input instanceof URL) {
    return input.href;
  }
  return input.url;
};

const notesFetch =
  (notes: unknown, config: unknown): typeof fetch =>
  async (input) => {
    const body = urlOf(input).includes('/config') ? config : notes;
    return Response.json(body);
  };

describe('notes list load', () => {
  it('exposes the error param and resolves the notes', async () => {
    const result = await runLoad(listLoad, {
      fetch: jsonFetch([{ id: 'n1' }]),
      url: new URL('http://x/notes?error=nope')
    });
    expect(result.error).toBe('nope');
    await expect(result.notes).resolves.toEqual([{ id: 'n1' }]);
  });

  it('falls back to an empty notes array when data is missing', async () => {
    const result = await runLoad(listLoad, {
      fetch: jsonFetch(null),
      url: new URL('http://x/notes')
    });
    expect(result.error).toBeNull();
    await expect(result.notes).resolves.toEqual([]);
  });

  it('resolves the notes config', async () => {
    const result = await runLoad(listLoad, {
      fetch: notesFetch([], { max_per_user: 20 }),
      url: new URL('http://x/notes')
    });
    await expect(result.notesConfig).resolves.toEqual({ max_per_user: 20 });
  });
});

describe('note create load', () => {
  it('resolves notes and config without redirecting', async () => {
    const result = await runLoad(createLoad, {
      fetch: notesFetch([], { max_per_user: 20 })
    });
    await expect(result.notes).resolves.toEqual([]);
    await expect(result.notesConfig).resolves.toEqual({ max_per_user: 20 });
  });
});

describe('note detail load', () => {
  it('passes the id through and resolves the note + users promises', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch({ title: 'Hi' }),
      params: { id: 'n1' },
      url: new URL('http://x/notes')
    });
    expect(result.id).toBe('n1');
    await expect(result.noteRes).resolves.toMatchObject({
      data: { title: 'Hi' }
    });
    await expect(result.usersPromise).resolves.toBeDefined();
  });

  it('exposes the error query param and resolves the snapshots data', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch([{ id: 's1' }]),
      params: { id: 'n1' },
      url: new URL('http://x/notes/n1?error=not_found')
    });
    expect(result.error).toBe('not_found');
    await expect(result.snapshotsPromise).resolves.toEqual([{ id: 's1' }]);
  });

  it('defaults the error to null and unwraps the snapshots payload', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch([]),
      params: { id: 'n1' },
      url: new URL('http://x/notes/n1')
    });
    expect(result.error).toBeNull();
    // `listNoteSnapshots(...).then(({ data }) => data)` unwraps the response to
    // The bare snapshots array.
    await expect(result.snapshotsPromise).resolves.toEqual([]);
  });
});

describe('snapshot detail load', () => {
  it('passes the id + snapshot through and resolves the snapshot info', async () => {
    const result = await runLoad(snapshotLoad, {
      fetch: jsonFetch({ note_id: 'n1', title: 'Snap' }),
      params: { id: 'n1', snapshot: 's1' }
    });
    expect(result.id).toBe('n1');
    expect(result.snapshot).toBe('s1');
    await expect(result.snapshotRes).resolves.toMatchObject({
      data: { note_id: 'n1', title: 'Snap' }
    });
  });

  it('surfaces a 404 response without throwing', async () => {
    const result = await runLoad(snapshotLoad, {
      fetch: jsonFetch({ message: 'missing' }, 404),
      params: { id: 'n1', snapshot: 'missing' }
    });
    const res = await result.snapshotRes;
    expect(res.data).toBeUndefined();
    expect(res.response?.status).toBe(404);
  });
});
