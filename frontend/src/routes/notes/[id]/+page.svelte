<script lang="ts">
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import Lock from '@lucide/svelte/icons/lock';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import {
    deleteNote,
    editNote,
    shareNote,
    type NoteInfo,
    type SimpleUserInfo,
    type UserInfo
  } from '$lib/client';
  import TipTab from '$lib/components/tiptap/TipTab.svelte';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import NoteShareControl from '$lib/components/notes/NoteShareControl.svelte';
  import { Input } from '@profidev/pleiades/components/ui/input';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let titleSaving = $state(false);
  let note: NoteInfo | undefined = $state();
  let users: SimpleUserInfo[] | undefined = $state();
  let readonly = $derived(!note?.is_owner);
  let shareSaving = $state(false);
  let title = $state('');
  let sharedWithUsers = $state<SimpleUserInfo[]>([]);
  let sharedUpdateTimeout: ReturnType<typeof setTimeout> | undefined =
    $state(undefined);
  let userInfo: UserInfo | undefined = $state(undefined);

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
      sharedWithUsers = res.data.shared_with;
    });
  });

  $effect(() => {
    data.usersPromise.then(({ data: userList }) => {
      users = userList;
    });
  });

  $effect(() => {
    data.user.then((userInfoData) => {
      userInfo = userInfoData;
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
      sharedWithUsers = note.shared_with;
      toast.error('Failed to update shared users');
    } else {
      toast.success('Shared users updated');
    }
  };

  const handleShareSelectChange = (selected: string[]) => {
    if (sharedUpdateTimeout) clearTimeout(sharedUpdateTimeout);
    sharedUpdateTimeout = setTimeout(() => {
      onSharedChange(selected);
    }, 500);
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
  <div class="mb-0 ml-7 flex min-w-0 items-center gap-2 md:m-0">
    <Button size="icon" variant="ghost" href="/notes" class="shrink-0">
      <ArrowLeft class="size-5" />
    </Button>

    <Input
      class="bg-background! mr-auto max-w-70 flex-1 border-none text-xl!"
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

    <NoteShareControl
      {shareableUsers}
      selected={sharedWithUsers}
      onSelectChange={handleShareSelectChange}
      {readonly}
      saving={shareSaving}
    />
    {#if note}
      <div
        class="flex h-9 shrink-0 cursor-default items-center gap-2 rounded-full border px-3.5 pl-1.5 text-sm font-medium"
        title="Owner can't be changed"
      >
        <UserAvatar
          userId={note.owner.id}
          username={note.owner.name}
          class="size-6.5"
        />
        <span class="max-w-32 truncate">{note.owner.name}</span>
        <Lock class="text-muted-foreground size-3.5 shrink-0" />
      </div>
    {/if}
    <Separator orientation="vertical" class="h-5" />

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
  <div class="flex min-h-0 grow flex-col space-y-4">
    <TipTab id={data.id} username={userInfo?.name} />
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
