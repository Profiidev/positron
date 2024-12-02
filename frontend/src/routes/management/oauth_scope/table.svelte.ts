import {
  Permission,
  type OAuthPolicyInfo,
  type OAuthScope,
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
): ColumnDef<OAuthScope>[] => [
  createColumn("name", "Name"),
  createColumn("scope", "Scope"),
  createColumn(
    "policy",
    "Policies",
    (policy: OAuthPolicyInfo[]) =>
      policy.map((p) => p.name).join(", ") || "No Policies",
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
