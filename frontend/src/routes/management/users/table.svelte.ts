import {
  getPermissionGroups,
  Permission,
  type User,
} from "$lib/backend/management/types.svelte";
import { remove_user } from "$lib/backend/management/user.svelte";
import FormDialog from "$lib/components/form/form-dialog.svelte";
import {
  createColumn,
  createColumnHeader,
} from "$lib/components/table/helpers.svelte";
import Multiselect from "$lib/components/table/multiselect.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import Avatar from "$lib/components/util/avatar.svelte";
import { DateTime } from "$lib/util/time.svelte";
import type { ColumnDef } from "@tanstack/table-core";
import { Trash } from "lucide-svelte";
import { createRawSnippet, mount, unmount } from "svelte";
import { toast } from "svelte-sonner";

export const columns = (
  allowed_permissions: Permission[],
  access_level: number,
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
        data: getPermissionGroups(),
        selected: row.getValue<Permission[]>("permissions"),
        filter: (value) =>
          allowed_permissions.includes(value.value as Permission),
        disabled:
          !allowed_permissions.includes(Permission.UserEdit) ||
          row.getValue<number>("access_level") <= access_level,
        onSelect,
        label: "permissions",
      });
    },
    ...createColumnHeader("permissions", "Permissions"),
  },
  createColumn("access_level", "Access Level"),
  createColumn("uuid", "Uuid"),
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
          class: "ml-auto",
          disabled:
            !allowed_permissions.includes(Permission.UserDelete) ||
            row.getValue<number>("access_level") <= access_level,
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
          let ret = await remove_user(row.getValue("uuid"));

          if (ret) {
            return "Error while deleting user";
          } else {
            await updateUser();
            toast.success("Deleted User");
          }
        },
      });
    },
    enableHiding: false,
  },
];
