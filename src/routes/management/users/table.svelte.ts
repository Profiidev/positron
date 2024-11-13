import { Permission, type User } from "$lib/backend/management/types.svelte";
import {
  createColumn,
  createColumnHeader,
} from "$lib/components/table/helpers.svelte";
import Multiselect from "$lib/components/table/multiselect.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import Avatar from "$lib/components/util/avatar.svelte";
import type { ColumnDef } from "@tanstack/table-core";
import { DateTime } from "luxon";

export const columns = (
  allowed_permissions: Permission[],
  priority: number,
  permission_select?: (user: string, value: Permission, add: boolean) => void,
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
  },
  createColumn("name", "Name"),
  createColumn("email", "Email"),
  createColumn("last_login", "Last Login", (date: string) => {
    return DateTime.fromISO(date)
      .setLocale("de")
      .toLocaleString(DateTime.DATETIME_SHORT);
  }),
  {
    cell: ({ row }) => {
      let onSelect;
      if (permission_select) {
        onSelect = (value: Permission, add: boolean) =>
          permission_select(row.getValue<string>("uuid"), value, add);
      }

      return renderComponent(Multiselect<Permission>, {
        data: Object.keys(Permission).map((p) => ({
          label: p.toString(),
          value: p as Permission,
        })),
        selected: row.getValue<Permission[]>("permissions"),
        filter: (value) =>
          allowed_permissions.includes(value.value as Permission),
        disabled:
          !allowed_permissions.includes(Permission.UserEdit) ||
          row.getValue<number>("priority") < priority,
        onSelect,
      });
    },
    ...createColumnHeader("permissions", "Permissions"),
  },
  createColumn("uuid", "Uuid"),
  createColumn("priority", "Priority"),
];
