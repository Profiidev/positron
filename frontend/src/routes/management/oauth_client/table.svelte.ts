import {
  Permission,
  type GroupInfo,
  type OAuthClientInfo,
  type UserInfo
} from '$lib/backend/management/types.svelte';
import Actions from 'positron-components/components/table/actions.svelte';
import { createColumn } from 'positron-components/components/table/helpers.svelte';
import * as DataTable from 'positron-components/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';

export const columns = (
  edit: (user: string) => void,
  remove: (user: string) => void,
  data: Permission[]
): ColumnDef<OAuthClientInfo>[] => [
  createColumn('name', 'Name'),
  createColumn('client_id', 'Client ID'),
  createColumn('redirect_uri', 'Redirect URI'),
  createColumn(
    'additional_redirect_uris',
    'Other Redirect URIs',
    (uris: string[]) => uris.join(', ') || 'No Additional URIs'
  ),
  createColumn(
    'default_scope',
    'Default Scope',
    (s: string) => s || 'No Scopes'
  ),
  createColumn(
    'group_access',
    'Groups',
    (groups: GroupInfo[]) => groups.map((g) => g.name).join(', ') || 'No Groups'
  ),
  createColumn(
    'user_access',
    'Users',
    (users: UserInfo[]) => users.map((u) => u.name).join(', ') || 'No Users'
  ),
  {
    accessorKey: 'actions',
    header: () => {},
    cell: ({ row }) => {
      return DataTable.renderComponent(Actions, {
        edit_disabled: !data.includes(Permission.OAuthClientEdit),
        delete_disabled: !data.includes(Permission.OAuthClientDelete),
        edit: () => edit(row.getValue('client_id')),
        remove: () => remove(row.getValue('client_id'))
      });
    },
    enableHiding: false
  }
];
