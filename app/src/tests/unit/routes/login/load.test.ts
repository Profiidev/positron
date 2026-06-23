import { describe, expect, it } from 'vitest';
import { load } from '$routes/login/+page';

const run = async (search: string) =>
  load({ url: new URL(`http://localhost/login${search}`) } as never);

describe('login load', () => {
  it('extracts code and redirect from the query', async () => {
    expect(await run('?code=abc&redirect=https://x.example.com')).toEqual({
      code: 'abc',
      redirect: 'https://x.example.com'
    });
  });

  it('returns undefined for absent params', async () => {
    expect(await run('')).toEqual({ code: undefined, redirect: undefined });
  });

  it('returns code only when redirect is absent', async () => {
    expect(await run('?code=abc')).toEqual({
      code: 'abc',
      redirect: undefined
    });
  });
});
