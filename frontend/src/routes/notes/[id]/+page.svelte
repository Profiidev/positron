<script lang="ts">
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import Multiselect from '@profidev/pleiades/components/table/multiselect.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import {
    deleteNote,
    editNote,
    shareNote,
    type NoteInfo,
    type SimpleUserInfo
  } from '$lib/client';
  import { Label } from '@profidev/pleiades/components/ui/label';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let titleSaving = $state(false);
  let note: NoteInfo | undefined = $state();
  let users: SimpleUserInfo[] | undefined = $state();
  let readonly = $derived(!note?.is_owner);
  let shareSaving = $state(false);
  let title = $state('');
  let sharedWithIds = $state<string[]>([]);
  let sharedUpdateTimeout: ReturnType<typeof setTimeout> | undefined =
    $state(undefined);

  let shareableUsers = $derived(
    users?.filter((user) => user.id !== note?.owner.id) ?? []
  );

  $effect(() => {
    data.noteRes.then((res) => {
      if (!res.data) {
        if (res.response?.status === 404) {
          goto('/notes?error=not_found');
        } else {
          goto('/notes?error=other');
        }
        return;
      }

      note = res.data;
      title = res.data.title;
      sharedWithIds = res.data.shared_with.map((user) => user.id);
    });
  });

  $effect(() => {
    data.usersPromise.then(({ data: userList }) => {
      users = userList;
    });
  });

  const saveTitle = async () => {
    if (!note || readonly) return;

    const trimmed = title.trim();
    if (!trimmed || trimmed === note.title) {
      title = note.title;
      return;
    }

    titleSaving = true;
    const res = await editNote({
      body: { note_id: note.id, title: trimmed }
    });
    titleSaving = false;

    if (res.error) {
      title = note.title;
      toast.error('Failed to update title');
    } else {
      note = { ...note, title: trimmed };
      title = trimmed;
      toast.success('Title updated');
    }
  };

  const onSharedChange = async (selected: string[]) => {
    if (!note || readonly) return;

    shareSaving = true;
    const res = await shareNote({
      body: {
        note_id: note.id,
        shared_with: selected
      }
    });
    shareSaving = false;

    if (res.error) {
      sharedWithIds = note.shared_with.map((user) => user.id);
      toast.error('Failed to update shared users');
    } else {
      note = {
        ...note,
        shared_with: shareableUsers.filter((user) => selected.includes(user.id))
      };
      toast.success('Shared users updated');
    }
  };

  const deleteItemConfirm = async () => {
    if (!note) return;
    isLoading = true;
    let ret = await deleteNote({
      body: { note_id: note.id }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete note' };
    } else {
      toast.success(`Note ${note.title} deleted successfully`);
      setTimeout(() => {
        goto('/notes');
      });
    }
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mt-1! mb-0 ml-7 flex min-w-0 items-center gap-2 md:m-0">
    <Button size="icon" variant="ghost" href="/notes" class="shrink-0">
      <ArrowLeft class="size-5" />
    </Button>
    <div class="flex min-w-0 flex-1">
      <Input
        class="max-w-70 flex-1"
        bind:value={title}
        placeholder="Note title"
        disabled={readonly || titleSaving}
        onblur={saveTitle}
        onkeydown={(event) => {
          if (event.key === 'Enter') {
            event.currentTarget.blur();
          }
        }}
      />
      <Label class="ml-2">Shared with:</Label>
      <Multiselect
        class="ml-2 max-w-70 flex-1"
        data={shareableUsers.map((user) => ({
          label: user.name,
          value: user.id
        }))}
        label="Shared with"
        selected={sharedWithIds}
        disabled={readonly || shareSaving}
        onSelectChange={(selected) => {
          if (sharedUpdateTimeout) clearTimeout(sharedUpdateTimeout);
          sharedUpdateTimeout = setTimeout(() => {
            onSharedChange(selected);
          }, 500);
        }}
      />
    </div>
    <Button
      class="shrink-0 cursor-pointer"
      onclick={() => (deleteOpen = true)}
      variant="destructive"
      disabled={readonly}
    >
      <Trash />
      Delete
    </Button>
  </div>
  <Separator class="my-4" />
  <div class="flex min-h-0 grow flex-col space-y-4">
    {#if note}
      <p class="text-muted-foreground text-sm">Owner: {note.owner.name}</p>
    {/if}
  </div>
</div>
<FormDialog
  title="Delete Note"
  description={`Do you really want to delete the note ${note?.title}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
