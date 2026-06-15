import { describe, expect, it } from 'vitest';
import { load } from '$routes/login/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

describe('login load', () => {
  it('short-circuits with the error when one is in the query', async () => {
    const result = await runLoad(load, {
      fetch: jsonFetch(null),
      url: new URL('http://x/login?error=boom')
    });
    expect(result).toEqual({ error: 'boom' });
  });

  it('loads the auth config when there is no error', async () => {
    const result = await runLoad(load, {
      fetch: jsonFetch({ providers: [] }),
      url: new URL('http://x/login')
    });
    expect(result.error).toBeUndefined();
    await expect(result.config).resolves.toEqual({ providers: [] });
  });
});
