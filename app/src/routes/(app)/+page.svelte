<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import * as ScrollArea from '@profidev/pleiades/components/ui/scroll-area';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import Plus from '@lucide/svelte/icons/plus';
  import Trash from '@lucide/svelte/icons/trash';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import Nav from '$lib/components/Nav.svelte';
  import { deleteNote, type NoteInfo } from '$lib/commands/notes.svelte';
  import { notesConfigState, notesState } from '$lib/updater/state.svelte';

  const notes = $derived(notesState.value ?? undefined);
  const maxPerUser = $derived(notesConfigState.value?.max_per_user);

  let selected: NoteInfo | undefined = $state();
  let deleteOpen = $state(false);
  let isLoading = $state(false);

  const ownedCount = $derived(notes?.filter((n) => n.is_owner).length ?? 0);
  const atNoteLimit = $derived(
    maxPerUser !== undefined && ownedCount >= maxPerUser
  );

  const startDeleteNote = (item: NoteInfo, event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    selected = item;
    deleteOpen = true;
  };

  const deleteItemConfirm = async () => {
    if (!selected) return;

    isLoading = true;
    const ok = await deleteNote(selected.id);
    isLoading = false;

    if (!ok) {
      return { error: 'Failed to delete note' };
    } else {
      toast.success(`Note ${selected.title} deleted successfully`);
    }
  };
</script>

<div class="flex min-h-0 flex-1 flex-col p-4 pt-1">
  <div class="flex items-center">
    <h3 class="text-xl font-medium">Notes</h3>
    <Button
      class="ml-auto cursor-pointer"
      href={atNoteLimit ? undefined : '/notes/create'}
      disabled={atNoteLimit}
      title={atNoteLimit ? `Note limit reached (${maxPerUser})` : undefined}
    >
      <Plus />
      Create
    </Button>
  </div>
  <ScrollArea.ScrollArea orientation="vertical" class="mt-4 min-h-0 grow">
    <div class="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-4">
      {#if notes}
        {#each notes as note (note.id)}
          <a href="/notes/{note.id}" class="block h-full">
            <Card.Root
              class="hover:bg-muted/50 flex h-full flex-col gap-1 transition-colors"
            >
              <Card.Header>
                <div class="flex items-start gap-2">
                  <Card.Title class="line-clamp-2 flex-1 text-base">
                    {note.title}
                  </Card.Title>
                  {#if note.is_owner}
                    <Button
                      size="icon"
                      variant="ghost"
                      class="text-muted-foreground hover:text-destructive size-8 shrink-0 cursor-pointer"
                      onclick={(event) => startDeleteNote(note, event)}
                    >
                      <Trash class="size-4" />
                    </Button>
                  {/if}
                </div>
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
<FormDialog
  title="Delete Note"
  description={`Do you really want to delete the note ${selected?.title}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
