import { describe, expect, it } from 'vitest';
import {
  formatData,
  scopeSettings
} from '$routes/oauth-scope/[uuid]/schema.svelte';
import type { OAuthScopeInfo } from '$lib/client';

describe('scopeSettings schema', () => {
  it('applies defaults when fields are absent', () => {
    const r = scopeSettings.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toEqual({ name: '', policies: [], scope: '' });
  });

  it('rejects explicitly empty name or scope', () => {
    expect(scopeSettings.safeParse({ name: '', scope: 's' }).success).toBe(
      false
    );
  });
});

describe('formatData', () => {
  it('maps the policy objects down to their uuids', () => {
    const scope = {
      name: 'read',
      policies: [{ uuid: 'p1' }, { uuid: 'p2' }],
      scope: 'read'
    } as unknown as OAuthScopeInfo;

    expect(formatData(scope).policies).toEqual(['p1', 'p2']);
    expect(formatData(scope).name).toBe('read');
  });
});
