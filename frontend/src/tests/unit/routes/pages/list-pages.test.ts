import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { Component } from 'svelte';
import Groups from '$routes/groups/+page.svelte';
import Notes from '$routes/notes/+page.svelte';
import Users from '$routes/users/+page.svelte';
import Clients from '$routes/oauth-client/+page.svelte';
import Scopes from '$routes/oauth-scope/+page.svelte';
import Policies from '$routes/oauth-policy/+page.svelte';

const pr = async <T>(v: T) => Promise.resolve(v);

// [component, heading, data]
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const pages: [string, Component<any>, string, unknown][] = [
  [
    'groups',
    Groups,
    'Groups',
    {
      admin_group: pr(undefined),
      error: null,
      groups: pr([]),
      user: pr(undefined)
    }
  ],
  ['notes', Notes, 'Notes', { error: null, notes: pr([]), notesConfig: pr({ max_per_user: 20 }) }],
  [
    'users',
    Users,
    'Users',
    { error: null, user: pr(undefined), users: pr([]) }
  ],
  [
    'oauth-client',
    Clients,
    'OAuth / Oidc Clients',
    { clients: pr([]), error: null, user: pr(undefined) }
  ],
  [
    'oauth-scope',
    Scopes,
    'OAuth / Oidc Scopes',
    { error: null, scopes: pr([]), user: pr(undefined) }
  ],
  [
    'oauth-policy',
    Policies,
    'OAuth / Oidc Policies',
    { error: null, policies: pr([]), user: pr(undefined) }
  ]
];

describe.each(pages)('%s list page', (_name, Cmp, heading, data) => {
  it('renders its heading with empty data', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    render(Cmp, { data } as any);
    expect(screen.getByText(heading)).toBeInTheDocument();
  });
});
