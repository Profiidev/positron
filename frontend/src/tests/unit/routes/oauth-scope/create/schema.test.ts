import { describe, expect, it } from 'vitest';
import { information } from '$routes/oauth-scope/create/schema.svelte';

describe('oauth-scope create information schema', () => {
  it('applies defaults when fields are absent', () => {
    const r = information.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toEqual({ name: '', policies: [], scope: '' });
  });

  it('rejects explicitly empty name or scope', () => {
    expect(information.safeParse({ name: '', scope: 's' }).success).toBe(false);
    expect(information.safeParse({ name: 'n', scope: '' }).success).toBe(false);
  });

  it('accepts a valid payload with policies', () => {
    expect(
      information.safeParse({ name: 'n', policies: ['a'], scope: 'read' })
        .success
    ).toBe(true);
  });
});
