import { afterEach, describe, expect, it, vi } from 'vitest';
import { openAppLoginDeepLink } from '$lib/backend/auth.svelte';

describe('openAppLoginDeepLink', () => {
  afterEach(() => vi.restoreAllMocks());

  // oxlint-disable-next-line consistent-function-scoping
  const captureLink = () => {
    const link = document.createElement('a');
    const click = vi.spyOn(link, 'click').mockImplementation(() => {});
    const realCreate = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) =>
      tag === 'a' ? link : realCreate(tag)
    );
    return { click, link };
  };

  it('builds a positron://login deep link with code and redirect', () => {
    const { link } = captureLink();

    openAppLoginDeepLink('the-code', 'https://app.example.com');

    const url = new URL(link.href);
    expect(url.protocol).toBe('positron:');
    expect(url.host).toBe('login');
    expect(url.searchParams.get('code')).toBe('the-code');
    expect(url.searchParams.get('redirect')).toBe('https://app.example.com');
  });

  it('clicks the generated anchor exactly once to trigger navigation', () => {
    const { click } = captureLink();

    openAppLoginDeepLink('c', 'r');

    expect(click).toHaveBeenCalledTimes(1);
  });

  it('url-encodes special characters in the parameters', () => {
    const { link } = captureLink();

    openAppLoginDeepLink('a b&c=d', 'https://x.test/cb?next=/a&b');

    const url = new URL(link.href);
    // Round-trips through URL parsing without losing data.
    expect(url.searchParams.get('code')).toBe('a b&c=d');
    expect(url.searchParams.get('redirect')).toBe(
      'https://x.test/cb?next=/a&b'
    );
    // Raw href is percent-encoded, not literally containing the separators.
    expect(link.href).toContain('code=a+b%26c%3Dd');
  });
});
