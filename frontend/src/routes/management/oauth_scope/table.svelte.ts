import {
  Permission,
  type OAuthPolicyInfo,
  type OAuthScope
} from '$lib/backend/management/types.svelte';
import Actions from '@profidev/pleiades/components/table/actions.svelte';
import { createColumn } from '@profidev/pleiades/components/table/helpers.svelte';
import * as DataTable from '@profidev/pleiades/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';

export const columns = (
  edit: (uuid: string) => void,
  remove: (uuid: string) => void,
  data: Permission[]
): ColumnDef<OAuthScope>[] => [
  createColumn('name', 'Name'),
  createColumn('scope', 'Scope'),
  createColumn(
    'policy',
    'Policies',
    (policy: OAuthPolicyInfo[]) =>
      policy.map((p) => p.name).join(', ') || 'No Policies'
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
