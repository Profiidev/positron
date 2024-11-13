import {
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  type ColumnDef,
  type ColumnFiltersState,
  type SortingState,
  type Table,
  type VisibilityState,
} from "@tanstack/table-core";
import {
  createSvelteTable,
  renderComponent,
  renderSnippet,
} from "../ui/data-table";
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

let columnVisibility = $state<VisibilityState>({});
let sorting = $state<SortingState>([]);
let columnFilters = $state<ColumnFiltersState>([]);

export const createTable = <C>(
  data: C[],
  columns: ColumnDef<C>[],
): Table<C> => {
  return createSvelteTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onColumnVisibilityChange: (updater) => {
      if (typeof updater === "function") {
        columnVisibility = updater(columnVisibility);
      } else {
        columnVisibility = updater;
      }
    },
    onSortingChange: (updater) => {
      if (typeof updater === "function") {
        sorting = updater(sorting);
      } else {
        sorting = updater;
      }
    },
    onColumnFiltersChange: (updater) => {
      if (typeof updater === "function") {
        columnFilters = updater(columnFilters);
      } else {
        columnFilters = updater;
      }
    },
    state: {
      get columnVisibility() {
        return columnVisibility;
      },
      get sorting() {
        return sorting;
      },
      get columnFilters() {
        return columnFilters;
      }
    },
  });
};
