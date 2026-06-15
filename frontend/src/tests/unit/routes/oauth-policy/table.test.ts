import { describe, expect, it, vi } from 'vitest';

vi.mock('@profidev/pleiades/components/ui/data-table', () => ({
  renderComponent: (component: unknown, props: unknown) => ({
    component,
    props
  })
}));

const { columns } = await import('$routes/oauth-policy/table.svelte');
import { Permission } from '$lib/permissions.svelte';
import type { UserInfo } from '$lib/client';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const actions = (cols: any[], original: unknown) =>
  cols.find((c) => c.accessorKey === 'actions').cell({ row: { original } })
    .props;

describe('oauth-policy table columns', () => {
  it('disables actions without a user', () => {
    const cols = columns({ deletePolicy: () => {} });
    expect(actions(cols, { uuid: 'p1' }).edit_disabled).toBe(true);
  });

  it('enables actions with oauth_policy:edit and links by uuid', () => {
    const user = {
      permissions: [Permission.OAUTH_POLICY_EDIT]
    } as unknown as UserInfo;
    const cols = columns({ deletePolicy: () => {}, user });
    const props = actions(cols, { uuid: 'p1' });
    expect(props.edit_disabled).toBe(false);
    expect(props.edit).toBe('/oauth-policy/p1');
  });
});
