import { describe, expect, it } from 'vitest';
import {
  HorizontalLayout,
  VerticalLayout
} from '$lib/commands/settings.svelte';
import { settings } from '$routes/(app)/settings/schema.svelte';

const valid = {
  height: 600,
  horizontal_layout: [HorizontalLayout.Center],
  vertical_layout: [VerticalLayout.Top],
  width: 800
};

describe('settings schema', () => {
  it('accepts a valid settings object', () => {
    expect(settings.safeParse(valid).success).toBe(true);
  });

  it('coerces numeric strings for width and height', () => {
    const result = settings.safeParse({
      ...valid,
      height: '700',
      width: '900'
    });
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.width).toBe(900);
      expect(result.data.height).toBe(700);
    }
  });

  it.each([
    ['width', 499],
    ['height', 499]
  ])('rejects %s below 500', (key, value) => {
    expect(settings.safeParse({ ...valid, [key]: value }).success).toBe(false);
  });

  it('rejects a non-integer width', () => {
    expect(settings.safeParse({ ...valid, width: 500.5 }).success).toBe(false);
  });

  it('rejects an invalid horizontal_layout value', () => {
    expect(
      settings.safeParse({ ...valid, horizontal_layout: ['Diagonal'] }).success
    ).toBe(false);
  });

  it('rejects an invalid vertical_layout value', () => {
    expect(
      settings.safeParse({ ...valid, vertical_layout: ['Diagonal'] }).success
    ).toBe(false);
  });

  it('rejects a missing field', () => {
    const { width: _width, ...rest } = valid;
    expect(settings.safeParse(rest).success).toBe(false);
  });

  it('allows an empty horizontal_layout array (no minimum enforced)', () => {
    // The single-select FormSelect always yields a one-element array, but the
    // Schema itself has no `.min(1)`, so an empty selection parses cleanly.
    expect(
      settings.safeParse({ ...valid, horizontal_layout: [] }).success
    ).toBe(true);
  });
});
