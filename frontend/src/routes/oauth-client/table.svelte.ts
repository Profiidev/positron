import type { ColumnDef } from '@tanstack/table-core';
import * as DataTable from '@profidev/pleiades/components/ui/data-table';
import { createColumn } from '@profidev/pleiades/components/table/helpers.svelte';
import { Permission } from '$lib/permissions.svelte';
import type {
  OAuthClientInfo,
  SimpleGroupInfo,
  SimpleOAuthScopeInfo,
  SimpleUserInfo,
  UserInfo
} from '$lib/client';
import Actions from '@profidev/pleiades/components/table/actions.svelte';

export const columns = ({
  deleteClient,
  user
}: {
  deleteClient: (client: OAuthClientInfo) => void;
  user?: UserInfo;
}): ColumnDef<OAuthClientInfo>[] => [
  createColumn('name', 'Name'),
  createColumn('client_id', 'Client ID'),
  createColumn('redirect_uri', 'Redirect URI'),
  createColumn(
    'additional_redirect_uris',
    'Other Redirect URIs',
    (uris: string[]) => uris.join(', ') || 'No Additional Redirect URIs'
  ),
  createColumn(
    'default_scope',
    'Default Scope',
    (s: SimpleOAuthScopeInfo[]) =>
      s.map((d) => d.name).join(', ') || 'No Scopes'
  ),
  createColumn(
    'group_access',
    'Groups',
    (groups: SimpleGroupInfo[]) =>
      groups.map((u) => u.name).join(', ') || 'No Groups'
  ),
  createColumn(
    'user_access',
    'Users',
    (users: SimpleUserInfo[]) =>
      users.map((u) => u.name).join(', ') || 'No Users'
  ),
  {
    accessorKey: 'actions',
    cell: ({ row }) => {
      const disabled = !user
        ? true
        : !user?.permissions.includes(Permission.OAUTH_CLIENT_EDIT);

      return DataTable.renderComponent(Actions, {
        delete_disabled: disabled,
        edit: `/oauth-client/${row.original.client_id}`,
        edit_disabled: disabled,
        remove: () => deleteClient(row.original)
      });
    },
    enableHiding: false,
    header: () => {}
  }
];
