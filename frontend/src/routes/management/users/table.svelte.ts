import { Permission, type User } from '$lib/backend/management/types.svelte';
import type { ColumnDef } from '@tanstack/table-core';
import { Actions, createColumn } from 'positron-components/components/table';
import { DataTable } from 'positron-components/components/ui';
import { SimpleAvatar } from 'positron-components/components/util';
import { DateTime } from 'positron-components/util';

export const columns = (
  edit: (user: string) => void,
  remove: (user: string) => void,
  data: {
    allowed_permissions: Permission[];
    access_level: number;
  }
): ColumnDef<User>[] => [
  {
    accessorKey: 'image',
    header: () => {},
    cell: ({ row }) => {
      return DataTable.renderComponent(SimpleAvatar, {
        src: row.getValue('image') as string,
        class: 'size-8'
      });
    },
    size: 10
  },
  createColumn('name', 'Name'),
  createColumn('email', 'Email'),
  createColumn('last_login', 'Last Login', (date: string) => {
    return DateTime.fromISO(date)
      .setLocale('de')
      .toLocaleString(DateTime.DATETIME_SHORT);
  }),
  createColumn(
    'permissions',
    'Permissions',
    (permissions: Permission[]) => permissions.join(', ') || 'No Permissions'
  ),
  createColumn('access_level', 'Access Level'),
  createColumn('uuid', 'Uuid'),
  {
    accessorKey: 'actions',
    header: () => {},
    cell: ({ row }) => {
      return DataTable.renderComponent(Actions, {
        edit_disabled:
          data.access_level <= row.getValue<number>('access_level') ||
          !data.allowed_permissions.includes(Permission.UserEdit),
        delete_disabled:
          data.access_level <= row.getValue<number>('access_level') ||
          !data.allowed_permissions.includes(Permission.UserDelete),
        edit: () => edit(row.getValue('uuid')),
        remove: () => remove(row.getValue('uuid'))
      });
    },
    enableHiding: false
  }
];
