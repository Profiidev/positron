import { describe, expect, it } from 'vitest';
import { information } from '$routes/notes/create/schema.svelte';

describe('note create information schema', () => {
  it('defaults title to an empty string when absent', () => {
    const r = information.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data?.title).toBe('');
  });

  it('rejects an explicitly empty title', () => {
    expect(information.safeParse({ title: '' }).success).toBe(false);
  });

  it('accepts a non-empty title', () => {
    expect(information.safeParse({ title: 'Note' }).success).toBe(true);
  });
});
