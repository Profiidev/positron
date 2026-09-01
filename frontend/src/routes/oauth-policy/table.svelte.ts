import { renderComponent } from '@tanstack/svelte-table';
import { type TableColumnDef, createColumn } from '@profidev/pleiades/components/table/helpers.svelte';
import { Permission } from '$lib/permissions.svelte';
import type {
  OAuthPolicyContent,
  OAuthPolicyInfo,
  UserInfo
} from '$lib/client';
import Actions from '@profidev/pleiades/components/table/actions.svelte';

export const columns = ({
  deletePolicy,
  user
}: {
  deletePolicy: (policy: OAuthPolicyInfo) => void;
  user?: UserInfo;
}): TableColumnDef<OAuthPolicyInfo>[] => [
  createColumn('name', 'Name'),
  createColumn('claim', 'Claim'),
  createColumn('default', 'Default Content'),
  createColumn(
    'content',
    'Group Mappings',
    (mappings: OAuthPolicyContent[]) =>
      mappings.map((u) => `${u.group_name}: ${u.content}`).join(', ') ||
      'No Mappings'
  ),
  createColumn('uuid', 'Uuid'),
  {
    accessorKey: 'actions',
    cell: ({ row }) => {
      const disabled = !user
        ? true
        : !user?.permissions.includes(Permission.OAUTH_POLICY_EDIT);

      return renderComponent(Actions, {
        delete_disabled: disabled,
        edit: `/oauth-policy/${row.original.uuid}`,
        edit_disabled: disabled,
        remove: () => deletePolicy(row.original)
      });
    },
    enableHiding: false,
    header: () => {}
  }
];
