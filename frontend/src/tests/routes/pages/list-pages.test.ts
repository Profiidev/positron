import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { Component } from 'svelte';
import Groups from '$routes/groups/+page.svelte';
import Notes from '$routes/notes/+page.svelte';
import Users from '$routes/users/+page.svelte';
import Clients from '$routes/oauth-client/+page.svelte';
import Scopes from '$routes/oauth-scope/+page.svelte';
import Policies from '$routes/oauth-policy/+page.svelte';

const P =  async <T,>(v: T) => Promise.resolve(v);

// [component, heading, data]
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const pages: [string, Component<any>, string, unknown][] = [
  [
    'groups',
    Groups,
    'Groups',
    { admin_group: P(undefined), error: null, groups: P([]), user: P(undefined) }
  ],
  ['notes', Notes, 'Notes', { error: null, notes: P([]) }],
  [
    'users',
    Users,
    'Users',
    { error: null, user: P(undefined), users: P([]) }
  ],
  [
    'oauth-client',
    Clients,
    'OAuth / Oidc Clients',
    { clients: P([]), error: null, user: P(undefined) }
  ],
  [
    'oauth-scope',
    Scopes,
    'OAuth / Oidc Scopes',
    { error: null, scopes: P([]), user: P(undefined) }
  ],
  [
    'oauth-policy',
    Policies,
    'OAuth / Oidc Policies',
    { error: null, policies: P([]), user: P(undefined) }
  ]
];

describe.each(pages)('%s list page', (_name, Cmp, heading, data) => {
  it('renders its heading with empty data', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    render(Cmp, { data } as any);
    expect(screen.getByText(heading)).toBeInTheDocument();
  });
});
