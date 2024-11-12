import type { ColumnDef } from "@tanstack/table-core";
import { renderComponent, renderSnippet } from "../ui/data-table";
import { createRawSnippet } from "svelte";
import TableHead from "./table-head.svelte";

export const createColumnHeader = <T, C>(
  key: string,
  title: string,
): ColumnDef<C> => {
  return {
    accessorKey: key,
    header: ({ column }) =>
      renderComponent(TableHead, {
        onclick: () => column.toggleSorting(column.getIsSorted() === "asc"),
        title,
      }),
  };
};

export const createColumnCell = <T, C>(
  key: string,
  formatter?: (value: T) => string,
): ColumnDef<C> => {
  return {
    accessorKey: key,
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
  };
};

export const createColumn = <T, C>(
  key: string,
  title: string,
  formatter?: (value: T) => string,
): ColumnDef<C> => {
  return {
    ...createColumnHeader(key, title),
    ...createColumnCell(key, formatter),
  };
};
