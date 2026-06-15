import { describe, expect, it, vi } from 'vitest';

vi.mock('@profidev/pleiades/components/ui/data-table', () => ({
  renderComponent: (component: unknown, props: unknown) => ({
    component,
    props
  })
}));

const { columns } = await import('$routes/oauth-client/table.svelte');
import { Permission } from '$lib/permissions.svelte';
import type { UserInfo } from '$lib/client';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const actions = (cols: any[], original: unknown) =>
  cols.find((c) => c.accessorKey === 'actions').cell({ row: { original } })
    .props;

describe('oauth-client table columns', () => {
  it('disables actions without a user', () => {
    const cols = columns({ deleteClient: () => {} });
    expect(actions(cols, { client_id: 'c1' }).edit_disabled).toBe(true);
  });

  it('disables actions without oauth_client:edit', () => {
    const user = { permissions: [] } as unknown as UserInfo;
    const cols = columns({ deleteClient: () => {}, user });
    expect(actions(cols, { client_id: 'c1' }).delete_disabled).toBe(true);
  });

  it('enables actions with oauth_client:edit and links by client_id', () => {
    const user = {
      permissions: [Permission.OAUTH_CLIENT_EDIT]
    } as unknown as UserInfo;
    const cols = columns({ deleteClient: () => {}, user });
    const props = actions(cols, { client_id: 'c1' });
    expect(props.edit_disabled).toBe(false);
    expect(props.edit).toBe('/oauth-client/c1');
  });
});
