import type { ColumnDef } from '@tanstack/table-core';
import * as DataTable from '@profidev/pleiades/components/ui/data-table';
import { createColumn } from '@profidev/pleiades/components/table/helpers.svelte';
import type { SessionInfo } from '$lib/client';
import { formatRelativeOptional, formatRelativePast } from './session-utils';
import SessionCell from './SessionCell.svelte';
import SessionTypeCell from './SessionTypeCell.svelte';
import ExpiresCell from './ExpiresCell.svelte';
import SessionActions from './SessionActions.svelte';

export const columns = ({
  revoke
}: {
  revoke: (session: SessionInfo) => void;
}): ColumnDef<SessionInfo>[] => [
  {
    accessorKey: 'session',
    cell: ({ row }) =>
      DataTable.renderComponent(SessionCell, { session: row.original }),
    header: () => 'Session'
  },
  {
    accessorKey: 'is_app',
    cell: ({ row }) =>
      DataTable.renderComponent(SessionTypeCell, {
        isApp: row.original.is_app
      }),
    header: () => 'Type'
  },
  createColumn('created_at', 'Created', (date: string) =>
    formatRelativePast(new Date(date))
  ),
  createColumn('last_used_at', 'Last used', (date: string) =>
    formatRelativePast(new Date(date))
  ),
  createColumn('refreshed_at', 'Last refresh', (date?: string) =>
    formatRelativeOptional(date ? new Date(date) : undefined)
  ),
  {
    accessorKey: 'expires_at',
    cell: ({ row }) =>
      DataTable.renderComponent(ExpiresCell, {
        expiresAt: new Date(row.original.expires_at)
      }),
    header: () => 'Expires'
  },
  {
    accessorKey: 'actions',
    cell: ({ row }) =>
      DataTable.renderComponent(SessionActions, {
        revoke,
        session: row.original
      }),
    enableHiding: false,
    header: () => {}
  }
];
