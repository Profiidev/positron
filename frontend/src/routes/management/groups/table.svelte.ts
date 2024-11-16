import { delete_group } from "$lib/backend/management/group.svelte";
import {
  getPermissionGroups,
  Permission,
  type Group,
  type UserInfo,
} from "$lib/backend/management/types.svelte";
import FormDialog from "$lib/components/form/form-dialog.svelte";
import {
  createColumn,
  createColumnHeader,
} from "$lib/components/table/helpers.svelte";
import Multiselect from "$lib/components/table/multiselect.svelte";
import { renderComponent } from "$lib/components/ui/data-table";
import type { ColumnDef } from "@tanstack/table-core";
import { Trash } from "lucide-svelte";
import { createRawSnippet, mount, unmount } from "svelte";
import { toast } from "svelte-sonner";

export const columns = (
  allowed_permissions: Permission[],
  possibleUsers: UserInfo[],
  access_level: number,
  updateGroups: () => Promise<void>,
  permission_select?: (group: string, value: Permission, add: boolean) => void,
  user_select?: (group: string, value: UserInfo, add: boolean) => void,
): ColumnDef<Group>[] => [
  createColumn("name", "Name"),
  createColumn("access_level", "Access Level"),
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
          !allowed_permissions.includes(Permission.GroupEdit) ||
          row.getValue<number>("access_level") <= access_level,
        onSelect,
        label: "permissions",
      });
    },
    ...createColumnHeader("permissions", "Permissions"),
  },
  {
    cell: ({ row }) => {
      let onSelect;
      if (user_select) {
        onSelect = (value: UserInfo, add: boolean) =>
          user_select(row.getValue<string>("uuid"), value, add);
      }

      return renderComponent(Multiselect<UserInfo>, {
        data: possibleUsers.map((u) => ({
          label: u.name,
          value: u,
        })),
        selected: row.getValue<UserInfo[]>("users"),
        disabled:
          !allowed_permissions.includes(Permission.GroupEdit) ||
          row.getValue<number>("access_level") <= access_level,
        onSelect,
        label: "users",
        display: (u) => u.name,
        compare: (a, b) => a.uuid === b.uuid,
      });
    },
    ...createColumnHeader("users", "Users"),
  },
  createColumn("uuid", "Uuid"),
  {
    accessorKey: "actions",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(FormDialog, {
        title: "Delete Group",
        description: `Do you really want to delete the group ${row.getValue("name")}`,
        confirm: "Delete",
        confirmVariant: "destructive",
        trigger: {
          size: "icon",
          variant: "destructive",
          class: "ml-auto",
          disabled:
            !allowed_permissions.includes(Permission.GroupDelete) ||
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
          let ret = await delete_group(row.getValue("uuid"));

          if (ret) {
            return "Error while deleting group";
          } else {
            await updateGroups();
            toast.success("Deleted Group");
          }
        },
      });
    },
    enableHiding: false,
  },
];
