import { describe, expect, it } from 'vitest';
import { isValidSearchPattern } from '$lib/components/tiptap/extensions/search-and-replace';

describe('isValidSearchPattern', () => {
  it('treats an empty search string as valid regardless of flags', () => {
    expect(isValidSearchPattern('', true, true)).toBe(true);
    expect(isValidSearchPattern('', true, false)).toBe(true);
    expect(isValidSearchPattern('', false, false)).toBe(true);
  });

  it('is always valid when regex mode is off (input is treated literally)', () => {
    expect(isValidSearchPattern('(', false, false)).toBe(true);
    expect(isValidSearchPattern('[a-z', false, true)).toBe(true);
    expect(isValidSearchPattern('a+b', false, false)).toBe(true);
  });

  it('accepts a well-formed regex when regex mode is on', () => {
    expect(isValidSearchPattern('a+b', true, false)).toBe(true);
    expect(isValidSearchPattern('[a-z]+', true, true)).toBe(true);
    expect(isValidSearchPattern('(foo|bar)', true, false)).toBe(true);
  });

  it('rejects a malformed regex when regex mode is on', () => {
    expect(isValidSearchPattern('(', true, false)).toBe(false);
    expect(isValidSearchPattern('[a-z', true, true)).toBe(false);
    expect(isValidSearchPattern('a{', true, false)).toBe(false);
  });

  it('does not let case sensitivity change validity of a valid pattern', () => {
    expect(isValidSearchPattern('abc', true, true)).toBe(
      isValidSearchPattern('abc', true, false)
    );
  });
});
