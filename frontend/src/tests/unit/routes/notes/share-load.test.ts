import { describe, expect, it } from 'vitest';
import { load as shareLoad } from '$routes/notes/share/[id]/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

describe('note public share load', () => {
  it('passes the id through and resolves the public note info', async () => {
    const result = await runLoad(shareLoad, {
      fetch: jsonFetch({
        can_edit: true,
        id: 'n1',
        owner: { id: 'u1', name: 'Owner' },
        title: 'Public note'
      }),
      params: { id: 'n1' }
    });

    expect(result.id).toBe('n1');
    await expect(result.noteRes).resolves.toMatchObject({
      data: {
        can_edit: true,
        id: 'n1',
        owner: { id: 'u1', name: 'Owner' },
        title: 'Public note'
      }
    });
  });

  it('surfaces an error response without throwing', async () => {
    const result = await runLoad(shareLoad, {
      fetch: jsonFetch({ message: 'note not found' }, 404),
      params: { id: 'missing' }
    });

    expect(result.id).toBe('missing');
    const res = await result.noteRes;
    expect(res.data).toBeUndefined();
    expect(res.error).toBeDefined();
  });
});
