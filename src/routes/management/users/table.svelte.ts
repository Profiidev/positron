import { Permission, type User } from "$lib/backend/management/types.svelte";
import { remove } from "$lib/backend/management/user.svelte";
import FormDialog from "$lib/components/form/form-dialog.svelte";
import {
  createColumn,
  createColumnHeader,
} from "$lib/components/table/helpers.svelte";
import Multiselect from "$lib/components/table/multiselect.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import Avatar from "$lib/components/util/avatar.svelte";
import type { ColumnDef } from "@tanstack/table-core";
import { Trash } from "lucide-svelte";
import { DateTime } from "luxon";
import { createRawSnippet, mount, unmount } from "svelte";

export const columns = (
  allowed_permissions: Permission[],
  priority: number,
  updateUser: () => Promise<void>,
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
    size: 10,
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
  {
    accessorKey: "actions",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(FormDialog, {
        title: "Delete User",
        description: `Do you really want to delete the user ${row.getValue("name")}`,
        confirm: "Delete",
        confirmVariant: "destructive",
        trigger: {
          size: "icon",
          variant: "destructive",
        },
        triggerInner: createRawSnippet<[]>(() => {
          return {
            render: () => "<div></div>",
            setup: (target) => {
              const comp = mount(Trash, { target, props: {} });
              return () => unmount(comp);
            },
          };
        }),
        onsubmit: async () => {
          let ret = await remove(row.getValue("uuid"));

          if (ret !== null) {
            return "Error while deleting user";
          } else {
            await updateUser();
          }
        },
      });
    },
  },
];
