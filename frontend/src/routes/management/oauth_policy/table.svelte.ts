import {
  Permission,
  type GroupInfo,
  type OAuthPolicy,
} from "$lib/backend/management/types.svelte";
import Actions from "$lib/components/table/actions.svelte";
import { createColumn } from "$lib/components/table/helpers.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import type { ColumnDef } from "@tanstack/table-core";

export const columns = (
  permissions: Permission[],
  access_level: number,
  edit: (uuid: string) => void,
  remove: (uuid: string) => void,
): ColumnDef<OAuthPolicy>[] => [
  createColumn("name", "Name"),
  createColumn("claim", "Claim"),
  createColumn("default", "Default Content"),
  createColumn(
    "group",
    "Group Mappings",
    (groups: [GroupInfo, string][]) =>
      groups.map((g) => `${g[0].name}: ${g[1]}`).join(", ") || "No Mappings",
  ),
  createColumn("uuid", "Uuid"),
  {
    accessorKey: "actions",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(Actions, {
        edit_disabled: !permissions.includes(Permission.OAuthClientEdit),
        delete_disabled: !permissions.includes(Permission.OAuthClientDelete),
        edit: () => edit(row.getValue("uuid")),
        remove: () => remove(row.getValue("uuid")),
      });
    },
    enableHiding: false,
  },
];
