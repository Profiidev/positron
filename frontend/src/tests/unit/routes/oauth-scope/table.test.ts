import { describe, expect, it, vi } from 'vitest';

vi.mock('@profidev/pleiades/components/ui/data-table', () => ({
  renderComponent: (component: unknown, props: unknown) => ({
    component,
    props
  })
}));

const { columns } = await import('$routes/oauth-scope/table.svelte');
import { Permission } from '$lib/permissions.svelte';
import type { UserInfo } from '$lib/client';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const actions = (cols: any[], original: unknown) =>
  cols.find((c) => c.accessorKey === 'actions').cell({ row: { original } })
    .props;

const editor = {
  permissions: [Permission.OAUTH_SCOPE_EDIT]
} as unknown as UserInfo;

describe('oauth-scope table columns', () => {
  it('disables actions without a user', () => {
    const cols = columns({ deleteScope: () => {} });
    expect(actions(cols, { scope: 'custom', uuid: 's1' }).edit_disabled).toBe(
      true
    );
  });

  it('enables actions for a custom scope with edit permission', () => {
    const cols = columns({ deleteScope: () => {}, user: editor });
    const props = actions(cols, { scope: 'custom', uuid: 's1' });
    expect(props.edit_disabled).toBe(false);
    expect(props.delete_disabled).toBe(false);
    expect(props.edit).toBe('/oauth-scope/s1');
  });

  it('blocks deletion of a built-in default scope even with permission', () => {
    const cols = columns({ deleteScope: () => {}, user: editor });
    const props = actions(cols, { scope: 'openid', uuid: 's1' });
    expect(props.edit_disabled).toBe(false);
    expect(props.delete_disabled).toBe(true);
  });
});
