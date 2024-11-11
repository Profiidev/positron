import type { Permission, User } from "$lib/backend/management/types.svelte";
import EmailButton from "$lib/components/table/email-button.svelte";
import {
  createSvelteTable,
  renderComponent,
  renderSnippet,
} from "$lib/components/ui/data-table";
import Avatar from "$lib/components/util/avatar.svelte";
import { getCoreRowModel, type ColumnDef } from "@tanstack/table-core";
import { DateTime } from "luxon";
import { createRawSnippet } from "svelte";

const createColumnHeader = (key: string, title: string): ColumnDef<User> => {
  return {
    accessorKey: key,
    header: ({ column }) =>
      renderComponent(EmailButton, {
        onclick: () => column.toggleSorting(column.getIsSorted() === "asc"),
        title,
      }),
  };
};

const createColumn = <T>(
  key: string,
  title: string,
  formatter?: (value: T) => string,
): ColumnDef<User> => {
  return {
    cell: ({ row }) => {
      const valueSnippet = createRawSnippet<[T]>((getValue) => {
        let value: string;
        if (formatter) {
          value = formatter(getValue());
        } else {
          value = getValue() as string;
        }

        return {
          render: () => `<div class="ml-4">${value}</div>`,
        };
      });

      return renderSnippet(valueSnippet, row.getValue(key));
    },
    ...createColumnHeader(key, title),
  };
};

export const columns: ColumnDef<User>[] = [
  {
    accessorKey: "image",
    header: () => {},
    cell: ({ row }) => {
      return renderComponent(Avatar, {
        src: row.getValue("image") as string,
        class: "size-8"
      });
    },
  },
  createColumn("name", "Name"),
  createColumn("email", "Email"),
  createColumn<string>("last_login", "Last Login", (date) => {
    return DateTime.fromISO(date).setLocale("de").toLocaleString(DateTime.DATETIME_SHORT);
  }),
  createColumn<Permission[]>("permissions", "Permissions", (permissions) => {
    return permissions.join(", ");
  }),
  createColumn("uuid", "Uuid"),
];

export const createTable = (data: User[]) =>
  createSvelteTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });
