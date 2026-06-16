import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/notes/+page';
import { load as createLoad } from '$routes/notes/create/+page';
import { load as detailLoad } from '$routes/notes/[id]/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

const notesFetch =
  (notes: unknown, config: unknown): typeof fetch =>
  async (input) => {
    const url = typeof input === 'string' ? input : input.url;
    const body = url.includes('/config') ? config : notes;
    return new Response(JSON.stringify(body), {
      headers: { 'content-type': 'application/json' }
    });
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
      params: { id: 'n1' }
    });
    expect(result.id).toBe('n1');
    await expect(result.noteRes).resolves.toMatchObject({
      data: { title: 'Hi' }
    });
    await expect(result.usersPromise).resolves.toBeDefined();
  });
});
