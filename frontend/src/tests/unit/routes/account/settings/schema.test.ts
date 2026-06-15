import { describe, expect, it } from 'vitest';
import { generalSettings } from '$routes/account/settings/schema.svelte';

describe('account settings schema', () => {
  it('defaults o_auth_instant_confirm to false', () => {
    const r = generalSettings.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data?.o_auth_instant_confirm).toBe(false);
  });

  it('accepts an explicit boolean', () => {
    const r = generalSettings.safeParse({ o_auth_instant_confirm: true });
    expect(r.data?.o_auth_instant_confirm).toBe(true);
  });
});
