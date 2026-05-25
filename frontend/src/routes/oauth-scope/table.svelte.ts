import type { ColumnDef } from '@tanstack/table-core';
import * as DataTable from '@profidev/pleiades/components/ui/data-table';
import { createColumn } from '@profidev/pleiades/components/table/helpers.svelte';
import { DEFAULT_SCOPES, Permission } from '$lib/permissions.svelte';
import type {
  OAuthScopeInfo,
  SimpleOAuthPolicyInfo,
  UserInfo
} from '$lib/client';
import Actions from '@profidev/pleiades/components/table/actions.svelte';

export const columns = ({
  deleteScope,
  user
}: {
  deleteScope: (scope: OAuthScopeInfo) => void;
  user?: UserInfo;
}): ColumnDef<OAuthScopeInfo>[] => [
  createColumn('name', 'Name'),
  createColumn('scope', 'Scope'),
  createColumn(
    'policies',
    'Policies',
    (policies: SimpleOAuthPolicyInfo[]) =>
      policies.map((u) => u.name).join(', ') || 'No Policies'
  ),
  createColumn('uuid', 'Uuid'),
  {
    accessorKey: 'actions',
    cell: ({ row }) => {
      const disabled = !user
        ? true
        : !user?.permissions.includes(Permission.OAUTH_SCOPE_EDIT);

      return DataTable.renderComponent(Actions, {
        delete_disabled:
          disabled || DEFAULT_SCOPES.includes(row.original.scope),
        edit: `/oauth-scope/${row.original.uuid}`,
        edit_disabled: disabled,
        remove: () => deleteScope(row.original)
      });
    },
    enableHiding: false,
    header: () => {}
  }
];
