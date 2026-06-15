import { afterEach, describe, expect, it, vi } from 'vitest';
import { extensions } from '$lib/components/tiptap/config';
import { getRandomColor } from '$lib/components/tiptap/color';

const PALETTE = [
  '#958DF1',
  '#F98181',
  '#FBBC88',
  '#FAF594',
  '#70CFF8',
  '#94FADB',
  '#B9F18D',
  '#FF85A2'
];

describe('extensions', () => {
  it('is a non-empty tiptap extension list', () => {
    expect(Array.isArray(extensions)).toBe(true);
    expect(extensions.length).toBeGreaterThan(0);
  });
});

describe('getRandomColor', () => {
  afterEach(() => vi.restoreAllMocks());

  it('always returns a colour from the palette', () => {
    for (let i = 0; i < 50; i += 1) {
      expect(PALETTE).toContain(getRandomColor());
    }
  });

  it('returns the first colour when random is 0', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0);
    expect(getRandomColor()).toBe(PALETTE[0]);
  });

  it('returns the last colour when random approaches 1', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0.999);
    expect(getRandomColor()).toBe(PALETTE[PALETTE.length - 1]);
  });
});
