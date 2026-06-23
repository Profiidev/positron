<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import * as ScrollArea from '@profidev/pleiades/components/ui/scroll-area';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import Nav from '$lib/components/Nav.svelte';
  import { listNotes, type NoteInfo } from '$lib/commands/notes.svelte';
  import { isConnected } from '$lib/updater/updater.svelte';

  let notes = $state<NoteInfo[] | undefined>();

  $effect(() => {
    // Refetch whenever the connection is (re)established.
    isConnected();
    listNotes().then((list) => {
      notes = list;
    });
  });
</script>

<div class="flex h-full flex-col">
  <Nav />
  <div class="flex min-h-0 flex-1 flex-col p-4 pt-0">
    <h3 class="text-xl font-medium">Notes</h3>
    <ScrollArea.ScrollArea orientation="vertical" class="mt-4 min-h-0 grow">
      <div class="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-4">
        {#if notes}
          {#each notes as note (note.id)}
            <a href="/notes/{note.id}" class="block h-full">
              <Card.Root
                class="hover:bg-muted/50 flex h-full flex-col gap-1 transition-colors"
              >
                <Card.Header>
                  <Card.Title class="line-clamp-2 text-base">
                    {note.title}
                  </Card.Title>
                  {#if note.shared_with.length > 0}
                    <Card.Description>
                      Shared with {note.shared_with
                        .map((user) => user.name)
                        .join(', ')}
                    </Card.Description>
                  {/if}
                </Card.Header>
                <Card.Content class="flex-1 pt-0">
                  <p class="text-muted-foreground line-clamp-4 text-sm">
                    {#if note.preview}
                      {note.preview}
                    {:else}
                      No content yet
                    {/if}
                  </p>
                </Card.Content>
              </Card.Root>
            </a>
          {:else}
            <div
              class="text-muted-foreground col-span-full flex h-32 items-center justify-center"
            >
              No notes yet
            </div>
          {/each}
        {:else}
          {#each Array(6) as _, index (index)}
            <Card.Root>
              <Card.Header>
                <Skeleton class="h-5 w-3/4" />
                <Skeleton class="h-4 w-1/2" />
              </Card.Header>
              <Card.Content>
                <Skeleton class="h-16 w-full" />
              </Card.Content>
            </Card.Root>
          {/each}
        {/if}
      </div>
    </ScrollArea.ScrollArea>
  </div>
</div>
