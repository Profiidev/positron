import { describe, expect, it } from 'vitest';
import { load as listLoad } from '$routes/oauth-client/+page';
import { load as createLoad } from '$routes/oauth-client/create/+page';
import { load as detailLoad } from '$routes/oauth-client/[uuid]/+page';
import { jsonFetch, runLoad } from '$test_helpers/load';

describe('oauth-client list load', () => {
  it('resolves clients and exposes the error param', async () => {
    const result = await runLoad(listLoad, {
      fetch: jsonFetch([{ client_id: 'c1' }]),
      url: new URL('http://x/oauth-client?error=e')
    });
    expect(result.error).toBe('e');
    await expect(result.clients).resolves.toEqual([{ client_id: 'c1' }]);
  });
});

describe('oauth-client create load', () => {
  it('resolves the available scopes', async () => {
    const result = await runLoad(createLoad, {
      fetch: jsonFetch([{ uuid: 's1' }])
    });
    await expect(result.scopes).resolves.toEqual([{ uuid: 's1' }]);
  });
});

describe('oauth-client detail load', () => {
  it('passes the uuid and resolves every dependent promise', async () => {
    const result = await runLoad(detailLoad, {
      fetch: jsonFetch({ name: 'app' }),
      params: { uuid: 'c1' }
    });
    expect(result.uuid).toBe('c1');
    await expect(result.clientRes).resolves.toMatchObject({
      data: { name: 'app' }
    });
    await expect(result.groupsPromise).resolves.toBeDefined();
    await expect(result.scopesPromise).resolves.toBeDefined();
    await expect(result.usersPromise).resolves.toBeDefined();
    await expect(result.sitePromise).resolves.toBeDefined();
  });
});
