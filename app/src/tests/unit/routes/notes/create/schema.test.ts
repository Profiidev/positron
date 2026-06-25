import { describe, expect, it } from 'vitest';
import { information } from '$routes/(app)/notes/create/schema.svelte';

describe('information schema', () => {
  it('accepts a non-empty title', () => {
    expect(information.safeParse({ title: 'My note' }).success).toBe(true);
  });

  it('rejects an empty title', () => {
    const result = information.safeParse({ title: '' });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Title is required');
    }
  });

  it('substitutes the empty-string default when title is omitted', () => {
    // Zod applies `.default('')` for a missing/undefined field and short-circuits
    // The min(1) check, so an omitted title parses to an empty string.
    const result = information.safeParse({});
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.title).toBe('');
    }
  });

  it('rejects a non-string title', () => {
    expect(information.safeParse({ title: 123 }).success).toBe(false);
  });

  it('keeps surrounding whitespace (no trim)', () => {
    const result = information.safeParse({ title: '  spaced  ' });
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.title).toBe('  spaced  ');
    }
  });
});
