import { describe, expect, it } from 'vitest';
import {
  clientSettings,
  formatData
} from '$routes/oauth-client/[uuid]/schema.svelte';
import type { OAuthClientInfo } from '$lib/client';

describe('clientSettings schema', () => {
  it('applies defaults when fields are absent', () => {
    const r = clientSettings.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toMatchObject({
      additional_redirect_uris: [],
      group_access: [],
      name: '',
      redirect_uri: '',
      require_pkce: false,
      scope: [],
      user_access: []
    });
  });

  it('rejects an explicitly empty name and an invalid redirect_uri', () => {
    expect(clientSettings.safeParse({ name: '' }).success).toBe(false);
    expect(
      clientSettings.safeParse({ name: 'a', redirect_uri: 'bad' }).success
    ).toBe(false);
  });
});

describe('formatData', () => {
  it('maps nested access objects down to their identifiers', () => {
    const client = {
      additional_redirect_uris: [],
      default_scope: [{ uuid: 's1' }, { uuid: 's2' }],
      group_access: [{ uuid: 'g1' }],
      name: 'app',
      redirect_uri: 'https://x/cb',
      require_pkce: false,
      user_access: [{ id: 'u1' }]
    } as unknown as OAuthClientInfo;

    const form = formatData(client);

    expect(form.group_access).toEqual(['g1']);
    expect(form.scope).toEqual(['s1', 's2']);
    expect(form.user_access).toEqual(['u1']);
    expect(form.name).toBe('app');
  });
});
