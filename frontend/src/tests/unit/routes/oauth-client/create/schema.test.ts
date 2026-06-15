import { describe, expect, it } from 'vitest';
import { information } from '$routes/oauth-client/create/schema.svelte';

describe('oauth-client create information schema', () => {
  it('applies defaults when fields are absent', () => {
    const r = information.safeParse({});
    expect(r.success).toBe(true);
    expect(r.data).toMatchObject({
      confidential: true,
      name: '',
      redirect_uri: '',
      require_pkce: false,
      scope: []
    });
  });

  it('rejects an explicitly empty name', () => {
    expect(information.safeParse({ name: '' }).success).toBe(false);
  });

  it('rejects an invalid redirect_uri', () => {
    expect(
      information.safeParse({ name: 'app', redirect_uri: 'not-a-url' }).success
    ).toBe(false);
  });

  it('accepts a full valid payload', () => {
    const r = information.safeParse({
      confidential: false,
      name: 'app',
      redirect_uri: 'https://example.com/cb',
      require_pkce: true,
      scope: ['openid']
    });
    expect(r.success).toBe(true);
  });
});
