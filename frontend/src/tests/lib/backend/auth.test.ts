import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  generateCodeChallenge,
  generateCodeVerifier
} from '$lib/backend/auth.svelte';

const URL_SAFE = /^[A-Za-z0-9\-._~]+$/;

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
