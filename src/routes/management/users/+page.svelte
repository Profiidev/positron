<script lang="ts">
  import type { User } from "$lib/backend/management/types.svelte";
  import { list } from "$lib/backend/management/user.svelte";
  import FlexRender from "$lib/components/ui/data-table/flex-render.svelte";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import * as Table from "$lib/components/ui/table";
  import { columns, createTable } from "./table.svelte";

  let users: User[] | undefined = $state();
  let users_promise = $state(list().then((user) => (users = user)));
  let table = $state(createTable([]));

  $effect(() => {
    if (users) {
      table = createTable(users);
    }
  });
</script>

<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Users</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete users and manage their permissions here
    </p>
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
            <Table.Cell colspan={columns.length} class="h-24 text-center">
              No results.
            </Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </ScrollArea>
</div>
