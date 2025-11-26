import {
  Permission,
  type GroupInfo,
  type OAuthPolicy
} from '$lib/backend/management/types.svelte';
import Actions from 'positron-components/components/table/actions.svelte';
import { createColumn } from 'positron-components/components/table/helpers.svelte';
import * as DataTable from 'positron-components/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';

export const columns = (
  edit: (uuid: string) => void,
  remove: (uuid: string) => void,
  data: Permission[]
): ColumnDef<OAuthPolicy>[] => [
  createColumn('name', 'Name'),
  createColumn('claim', 'Claim'),
  createColumn('default', 'Default Content'),
  createColumn(
    'group',
    'Group Mappings',
    (groups: [GroupInfo, string][]) =>
      groups.map((g) => `${g[0].name}: ${g[1]}`).join(', ') || 'No Mappings'
  ),
  createColumn('uuid', 'Uuid'),
  {
    accessorKey: 'actions',
    header: () => {},
    cell: ({ row }) => {
      return DataTable.renderComponent(Actions, {
        edit_disabled: !data.includes(Permission.OAuthClientEdit),
        delete_disabled: !data.includes(Permission.OAuthClientDelete),
        edit: () => edit(row.getValue('uuid')),
        remove: () => remove(row.getValue('uuid'))
      });
    },
    enableHiding: false
  }
];
