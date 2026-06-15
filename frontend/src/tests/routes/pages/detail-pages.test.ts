import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import GroupDetail from '$routes/groups/[uuid]/+page.svelte';
import PolicyDetail from '$routes/oauth-policy/[uuid]/+page.svelte';
import ScopeDetail from '$routes/oauth-scope/[uuid]/+page.svelte';
import ClientDetail from '$routes/oauth-client/[uuid]/+page.svelte';
import UserDetail from '$routes/users/[uuid]/+page.svelte';

const P =  async <T,>(v: T) => Promise.resolve(v);
const ok =  async <T,>(d: T) => P({ data: d, response: { status: 200 } });
const user = P({
  email: 'a@b.com',
  name: 'Admin',
  permissions: [],
  totp_enabled: false,
  uuid: 'me'
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mounts = (Cmp: any, data: unknown) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { container } = render(Cmp, { data } as any);
  expect(container.innerHTML.length).toBeGreaterThan(0);
};

describe('detail [uuid] pages mount with resolved data', () => {
  it('group detail', () => {
    mounts(GroupDetail, {
      groupRes: P({
        data: {
          admin_group: 'a',
          group: { name: 'g', permissions: [], users: [] }
        }
      }),
      user,
      usersPromise: ok([]),
      uuid: 'g1'
    });
  });

  it('oauth-policy detail', () => {
    mounts(PolicyDetail, {
      groupsPromise: P([]),
      policyRes: P({
        data: { claim: 'c', content: [], default: 'd', name: 'p' }
      }),
      user,
      uuid: 'p1'
    });
  });

  it('oauth-scope detail', () => {
    mounts(ScopeDetail, {
      policiesPromise: P([]),
      scopeRes: P({ data: { name: 's', policies: [], scope: 's' } }),
      user,
      uuid: 's1'
    });
  });

  it('oauth-client detail', () => {
    mounts(ClientDetail, {
      clientRes: P({
        data: {
          additional_redirect_uris: [],
          default_scope: [],
          group_access: [],
          name: 'c',
          redirect_uri: '',
          require_pkce: false,
          user_access: []
        }
      }),
      groupsPromise: P([]),
      scopesPromise: P([]),
      secret: undefined,
      sitePromise: P(''),
      user,
      usersPromise: P([]),
      uuid: 'c1'
    });
  });

  it('user detail', () => {
    mounts(UserDetail, {
      groupsPromise: P([]),
      mailActivePromise: P(false),
      user,
      userInfoPromise: ok({ groups: [], name: 'u', permissions: [] }),
      uuid: 'u1'
    });
  });
});
