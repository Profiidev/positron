import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  generateCodeChallenge,
  generateCodeVerifier,
  openAppLoginDeepLink
} from '$lib/backend/auth.svelte';

const URL_SAFE = /^[A-Za-z0-9\-._~]+$/;

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

describe('generateCodeVerifier', () => {
  afterEach(() => vi.restoreAllMocks());

  it('defaults to 64 characters', () => {
    expect(generateCodeVerifier()).toHaveLength(64);
  });

  it('honours a custom length', () => {
    expect(generateCodeVerifier(16)).toHaveLength(16);
    expect(generateCodeVerifier(128)).toHaveLength(128);
  });

  it('only emits URL-safe characters', () => {
    for (let i = 0; i < 20; i += 1) {
      expect(generateCodeVerifier(64)).toMatch(URL_SAFE);
    }
  });

  it('maps each random value through the charset (index 0 → "A")', () => {
    vi.spyOn(crypto, 'getRandomValues').mockImplementation((arr) => {
      (arr as Uint32Array).fill(0);
      return arr;
    });
    expect(generateCodeVerifier(5)).toBe('AAAAA');
  });
});

describe('generateCodeChallenge', () => {
  it('produces the RFC 7636 reference challenge for the reference verifier', async () => {
    // RFC 7636 Appendix B test vector.
    const verifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
    const challenge = await generateCodeChallenge(verifier);
    expect(challenge).toBe('E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM');
  });

  it('is deterministic and URL-safe (no padding)', async () => {
    const a = await generateCodeChallenge('hello-world');
    const b = await generateCodeChallenge('hello-world');
    expect(a).toBe(b);
    expect(a).not.toMatch(/[+/=]/);
    expect(a).toMatch(URL_SAFE);
  });
});
