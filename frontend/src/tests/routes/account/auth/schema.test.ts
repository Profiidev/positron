import { describe, expect, it } from 'vitest';
import {
  emailChangeSchema,
  passkeyCreateSchema,
  passkeyDeleteSchema,
  passkeyEditSchema,
  passwordChange,
  totpAdd,
  totpRemove
} from '$routes/account/auth/schema.svelte';

describe('passkey schemas', () => {
  it('requires a non-empty name on create', () => {
    expect(passkeyCreateSchema.safeParse({ name: 'yk' }).success).toBe(true);
    expect(passkeyCreateSchema.safeParse({ name: '' }).success).toBe(false);
  });

  it('requires a name on edit and defaults the phantom field', () => {
    const r = passkeyEditSchema.safeParse({ name: 'yk' });
    expect(r.success).toBe(true);
    expect(passkeyEditSchema.safeParse({ name: '' }).success).toBe(false);
  });

  it('accepts an empty object for delete', () => {
    expect(passkeyDeleteSchema.safeParse({}).success).toBe(true);
  });
});

describe('passwordChange schema', () => {
  it('requires both fields', () => {
    expect(
      passwordChange.safeParse({ password: 'a', password_confirm: 'a' }).success
    ).toBe(true);
    expect(
      passwordChange.safeParse({ password: '', password_confirm: 'a' }).success
    ).toBe(false);
    expect(
      passwordChange.safeParse({ password: 'a', password_confirm: '' }).success
    ).toBe(false);
  });
});

describe('totp schemas', () => {
  it('requires a six character code to add', () => {
    expect(totpAdd.safeParse({ code: '123456' }).success).toBe(true);
    expect(totpAdd.safeParse({ code: '123' }).success).toBe(false);
  });

  it('accepts an empty object to remove (phantom optional)', () => {
    expect(totpRemove.safeParse({}).success).toBe(true);
  });
});

describe('emailChangeSchema superRefine', () => {
  it('does not require codes when entering a new email (email_input true)', () => {
    const r = emailChangeSchema.safeParse({
      email: 'a@b.com',
      email_input: true,
      new_code: '',
      old_code: ''
    });
    expect(r.success).toBe(true);
  });

  it('requires both six-character codes when confirming (email_input false)', () => {
    const r = emailChangeSchema.safeParse({
      email: 'a@b.com',
      email_input: false,
      new_code: '',
      old_code: ''
    });
    expect(r.success).toBe(false);
    const paths = r.error?.issues.map((i) => i.path[0]);
    expect(paths).toContain('new_code');
    expect(paths).toContain('old_code');
  });

  it('flags a code that is not exactly six characters', () => {
    const r = emailChangeSchema.safeParse({
      email: 'a@b.com',
      email_input: false,
      new_code: '12345',
      old_code: '123456'
    });
    expect(r.success).toBe(false);
    expect(r.error?.issues.map((i) => i.path[0])).toEqual(['new_code']);
  });

  it('passes when both codes are exactly six characters', () => {
    const r = emailChangeSchema.safeParse({
      email: 'a@b.com',
      email_input: false,
      new_code: '111111',
      old_code: '222222'
    });
    expect(r.success).toBe(true);
  });
});
