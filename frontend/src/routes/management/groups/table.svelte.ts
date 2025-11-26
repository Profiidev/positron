import {
  Permission,
  type Group,
  type UserInfo
} from '$lib/backend/management/types.svelte';
import Actions from 'positron-components/components/table/actions.svelte';
import { createColumn } from 'positron-components/components/table/helpers.svelte';
import * as DataTable from 'positron-components/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';

export const columns = (
  edit: (user: string) => void,
  remove: (user: string) => void,
  data: {
    allowed_permissions: Permission[];
    access_level: number;
  }
): ColumnDef<Group>[] => [
  createColumn('name', 'Name'),
  createColumn('access_level', 'Access Level'),
  createColumn(
    'permissions',
    'Permissions',
    (permissions: Permission[]) => permissions.join(', ') || 'No Permissions'
  ),
  createColumn(
    'users',
    'Users',
    (users: UserInfo[]) => users.map((u) => u.name).join(', ') || 'No Users'
  ),
  createColumn('uuid', 'Uuid'),
  {
    accessorKey: 'actions',
    header: () => {},
    cell: ({ row }) => {
      return DataTable.renderComponent(Actions, {
        edit_disabled:
          data.access_level <= row.getValue<number>('access_level') ||
          !data.allowed_permissions.includes(Permission.GroupEdit),
        delete_disabled:
          data.access_level <= row.getValue<number>('access_level') ||
          !data.allowed_permissions.includes(Permission.GroupDelete),
        edit: () => edit(row.getValue('uuid')),
        remove: () => remove(row.getValue('uuid'))
      });
    },
    enableHiding: false
  }
];
