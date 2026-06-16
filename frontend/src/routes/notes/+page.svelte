<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import * as ScrollArea from '@profidev/pleiades/components/ui/scroll-area';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import Plus from '@lucide/svelte/icons/plus';
  import Trash from '@lucide/svelte/icons/trash';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { invalidate } from '$app/navigation';
  import { deleteNote, type NoteInfo } from '$lib/client';

  const { data } = $props();

  let notes: NoteInfo[] | undefined = $state();
  let maxPerUser: number | undefined = $state();
  let selected: NoteInfo | undefined = $state();
  let deleteOpen = $state(false);
  let isLoading = $state(false);

  $effect(() => {
    data.notes.then((list) => {
      notes = list;
    });
  });

  $effect(() => {
    data.notesConfig.then((config) => {
      maxPerUser = config?.max_per_user;
    });
  });

  const ownedCount = $derived(notes?.filter((n) => n.is_owner).length ?? 0);
  const atNoteLimit = $derived(
    maxPerUser !== undefined && ownedCount >= maxPerUser
  );

  $effect(() => {
    if (data.error) {
      if (data.error === 'not_found') {
        toast.error('Note not found');
      } else if (data.error === 'other') {
        toast.error('Failed to load note');
      }

      const url = new URL(window.location.href);
      url.searchParams.delete('error');
      window.history.replaceState({}, '', url);
    }
  });

  const deleteItemConfirm = async () => {
    if (!selected) return;

    isLoading = true;
    let ret = await deleteNote({
      body: {
        note_id: selected.id
      }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete note' };
    } else {
      toast.success(`Note ${selected.title} deleted successfully`);
      invalidate((url) => url.pathname.startsWith('/api/notes/management'));
    }
  };

  const startDeleteNote = (item: NoteInfo, event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    selected = item;
    deleteOpen = true;
  };
</script>

<div class="flex max-h-screen flex-col p-4">
  <div class="ml-7 flex items-center md:m-0">
    <h3 class="text-xl font-medium">Notes</h3>
    <Button
      class="ml-auto cursor-pointer"
      href={atNoteLimit ? undefined : '/notes/create'}
      disabled={atNoteLimit}
      title={atNoteLimit
        ? `Note limit reached (${maxPerUser})`
        : undefined}
    >
      <Plus />
      Create
    </Button>
  </div>
  <ScrollArea.ScrollArea orientation="vertical" class="mt-4 min-h-0 grow">
    <div
      class="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-4 pr-4"
    >
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
