import { Permission, type User } from "$lib/backend/management/types.svelte";
import Actions from "$lib/components/table/actions.svelte";
import { createColumn } from "$lib/components/table/helpers.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import Avatar from "$lib/components/util/avatar.svelte";
import { DateTime } from "$lib/util/time.svelte";
import type { ColumnDef } from "@tanstack/table-core";

export const columns = (
  allowed_permissions: Permission[],
  access_level: number,
  edit: (user: string) => void,
  remove: (user: string) => void,
): ColumnDef<User>[] => [
  {
    accessorKey: "image",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(Avatar, {
        src: row.getValue("image") as string,
        class: "size-8",
      });
    },
    size: 10,
  },
  createColumn("name", "Name"),
  createColumn("email", "Email"),
  createColumn("last_login", "Last Login", (date: string) => {
    return DateTime.fromISO(date)
      .setLocale("de")
      .toLocaleString(DateTime.DATETIME_SHORT);
  }),
  createColumn(
    "permissions",
    "Permissions",
    (permissions: Permission[]) => permissions.join(", ") || "No Permissions",
  ),
  createColumn("access_level", "Access Level"),
  createColumn("uuid", "Uuid"),
  {
    accessorKey: "actions",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(Actions, {
        edit_disabled:
          access_level <= row.getValue<number>("access_level") ||
          !allowed_permissions.includes(Permission.UserEdit),
        delete_disabled:
          access_level <= row.getValue<number>("access_level") ||
          !allowed_permissions.includes(Permission.UserDelete),
        edit: () => edit(row.getValue("uuid")),
        remove: () => remove(row.getValue("uuid")),
      });
    },
    enableHiding: false,
  },
];
