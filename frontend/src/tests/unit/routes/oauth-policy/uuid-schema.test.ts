import { describe, expect, it } from 'vitest';
import {
  formatData,
  policySettings
} from '$routes/oauth-policy/[uuid]/schema.svelte';
import type { OAuthPolicyInfo } from '$lib/client';

describe('policySettings schema', () => {
  it('defaults claim, default and name to empty strings', () => {
    const r = policySettings.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toEqual({ claim: '', default: '', name: '' });
  });

  it('rejects an explicitly empty required field', () => {
    expect(
      policySettings.safeParse({ claim: 'c', default: 'd', name: '' }).success
    ).toBe(false);
  });
});

describe('formatData', () => {
  it('passes the policy through unchanged', () => {
    const policy = {
      claim: 'groups',
      default: 'user',
      name: 'p'
    } as unknown as OAuthPolicyInfo;
    expect(formatData(policy)).toEqual(policy);
  });
});
