<script lang="ts">
  import * as Table from "$lib/components/ui/table";
  import { FlexRender } from "../ui/data-table";
  import { Input } from "../ui/input";
  import { ScrollArea } from "../ui/scroll-area";
  import type { Table as TableType } from "@tanstack/table-core";

  type T = $$Generic;

  interface Props {
    table: TableType<T>;
  }

  let { table }: Props = $props();
</script>

<div class="w-full">
  <div class="flex items-center py-4">
    <Input
      placeholder="Filter entries"
      value={(table.getColumn("email")?.getFilterValue() as string) ?? ""}
      oninput={(e) =>
        table.getColumn("email")?.setFilterValue(e.currentTarget.value)}
      onchange={(e) =>
        table.getColumn("email")?.setFilterValue(e.currentTarget.value)}
      class="max-w-sm"
    />
  </div>
  <ScrollArea class="rounded-md border grid" orientation="both">
    <Table.Root class="table-fixed min-w-[1000px]">
      <Table.Header>
        {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row>
            {#each headerGroup.headers as header (header.id)}
              <Table.Head>
                {#if !header.isPlaceholder}
                  <FlexRender
                    content={header.column.columnDef.header}
                    context={header.getContext()}
                  />
                {/if}
              </Table.Head>
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body>
        {#each table.getRowModel().rows as row (row.id)}
          <Table.Row data-state={row.getIsSelected() && "selected"}>
            {#each row.getVisibleCells() as cell (cell.id)}
              <Table.Cell>
                <FlexRender
                  content={cell.column.columnDef.cell}
                  context={cell.getContext()}
                />
              </Table.Cell>
            {/each}
          </Table.Row>
        {:else}
          <Table.Row>
            <Table.Cell
              colspan={table.getAllColumns().length}
              class="h-24 text-center"
            >
              No results.
            </Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </ScrollArea>
</div>
