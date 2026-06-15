import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/oauth-scope/+page';
import { load as createLoad } from '$routes/oauth-scope/create/+page';
import { load as detailLoad } from '$routes/oauth-scope/[uuid]/+page';
import { jsonFetch, runLoad } from '../../_helpers/load';

describe('oauth-scope list load', () => {
  it('resolves scopes and exposes the error param', async () => {
    const result = await runLoad(listLoad, {
      fetch: jsonFetch([{ uuid: 's1' }]),
      url: new URL('http://x/oauth-scope?error=e')
    });
    expect(result.error).toBe('e');
    await expect(result.scopes).resolves.toEqual([{ uuid: 's1' }]);
  });
});

describe('oauth-scope create load', () => {
  it('resolves the available policies', async () => {
    const result = await runLoad(createLoad, {
      fetch: jsonFetch([{ uuid: 'p1' }])
    });
    await expect(result.policies).resolves.toEqual([{ uuid: 'p1' }]);
  });
});

describe('oauth-scope detail load', () => {
  it('passes the uuid and resolves the scope + policies promises', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch({ name: 's' }),
      params: { uuid: 's1' }
    });
    expect(result.uuid).toBe('s1');
    await expect(result.scopeRes).resolves.toMatchObject({
      data: { name: 's' }
    });
    await expect(result.policiesPromise).resolves.toBeDefined();
  });
});
