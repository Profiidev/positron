import { describe, expect, it } from 'vitest';
import { instanceUrl } from '$routes/setup/schema.svelte';

describe('instanceUrl schema', () => {
  it('accepts a valid https url', () => {
    expect(
      instanceUrl.safeParse({ url: 'https://positron.example.com' }).success
    ).toBe(true);
  });

  it('accepts a valid http url', () => {
    expect(
      instanceUrl.safeParse({ url: 'http://localhost:8080' }).success
    ).toBe(true);
  });

  it.each([
    ['empty string', ''],
    ['not a url', 'positron'],
    ['missing scheme', 'example.com']
  ])('rejects %s', (_label, url) => {
    expect(instanceUrl.safeParse({ url }).success).toBe(false);
  });

  it('rejects a missing url field', () => {
    expect(instanceUrl.safeParse({}).success).toBe(false);
  });

  it('rejects a non-string url', () => {
    expect(instanceUrl.safeParse({ url: 123 }).success).toBe(false);
  });
});
