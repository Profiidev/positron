import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/notes/+page';
import { load as detailLoad } from '$routes/notes/[id]/+page';
import { jsonFetch, runLoad } from '../../_helpers/load';

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
});

describe('note detail load', () => {
  it('passes the id through and resolves the note + users promises', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch({ title: 'Hi' }),
      params: { id: 'n1' }
    });
    expect(result.id).toBe('n1');
    await expect(result.noteRes).resolves.toMatchObject({
      data: { title: 'Hi' }
    });
    await expect(result.usersPromise).resolves.toBeDefined();
  });
});
