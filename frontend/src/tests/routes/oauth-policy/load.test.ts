import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/oauth-policy/+page';
import { load as detailLoad } from '$routes/oauth-policy/[uuid]/+page';
import { jsonFetch, runLoad } from '../../_helpers/load';

describe('oauth-policy list load', () => {
  it('resolves policies and exposes the error param', async () => {
    const result = await runLoad(listLoad, {
      fetch: jsonFetch([{ uuid: 'p1' }]),
      url: new URL('http://x/oauth-policy?error=e')
    });
    expect(result.error).toBe('e');
    await expect(result.policies).resolves.toEqual([{ uuid: 'p1' }]);
  });
});

describe('oauth-policy detail load', () => {
  it('passes the uuid and resolves the policy + groups promises', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch({ name: 'p' }),
      params: { uuid: 'p1' }
    });
    expect(result.uuid).toBe('p1');
    await expect(result.policyRes).resolves.toMatchObject({
      data: { name: 'p' }
    });
    await expect(result.groupsPromise).resolves.toBeDefined();
  });
});
