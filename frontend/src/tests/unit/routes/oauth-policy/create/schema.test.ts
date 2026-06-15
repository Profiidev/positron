import { describe, expect, it } from 'vitest';
import { information } from '$routes/oauth-policy/create/schema.svelte';

describe('oauth-policy create information schema', () => {
  it('defaults claim, default and name to empty strings when absent', () => {
    const r = information.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toEqual({ claim: '', default: '', name: '' });
  });

  it('rejects an explicitly empty required field', () => {
    expect(
      information.safeParse({ claim: '', default: 'd', name: 'n' }).success
    ).toBe(false);
    expect(
      information.safeParse({ claim: 'c', default: '', name: 'n' }).success
    ).toBe(false);
    expect(
      information.safeParse({ claim: 'c', default: 'd', name: '' }).success
    ).toBe(false);
  });

  it('accepts a fully populated payload', () => {
    expect(
      information.safeParse({ claim: 'groups', default: 'user', name: 'p' })
        .success
    ).toBe(true);
  });
});
