import { describe, expect, it } from 'vitest';
import { login, totpSchema } from '$routes/login/schema.svelte';

describe('login schema', () => {
  it('accepts a valid email and password', () => {
    expect(
      login.safeParse({ email: 'a@b.com', password: 'pw' }).success
    ).toBe(true);
  });

  it('rejects an invalid email', () => {
    const r = login.safeParse({ email: 'not-an-email', password: 'pw' });
    expect(r.success).toBe(false);
  });

  it('rejects an empty password', () => {
    const r = login.safeParse({ email: 'a@b.com', password: '' });
    expect(r.success).toBe(false);
  });
});

describe('totpSchema', () => {
  it('accepts exactly six characters', () => {
    expect(totpSchema.safeParse({ code: '123456' }).success).toBe(true);
  });

  it('rejects fewer than six characters', () => {
    expect(totpSchema.safeParse({ code: '12345' }).success).toBe(false);
  });

  it('rejects more than six characters', () => {
    expect(totpSchema.safeParse({ code: '1234567' }).success).toBe(false);
  });
});
