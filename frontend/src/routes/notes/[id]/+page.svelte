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
    deleteNoteSnapshot,
    editNote,
    shareNote,
    shareNotePublic,
    transferNote,
    type NoteInfo,
    type NoteShareAccess,
    type NoteSnapshotInfo,
    type SharedUserInfo,
    type SimpleUserInfo,
    type UserInfo
  } from '$lib/client';
  import TipTab from '$lib/components/tiptap/TipTab.svelte';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import NoteShareControl from '$lib/components/notes/NoteShareControl.svelte';
  import NoteTransferOwnerControl from '$lib/components/notes/NoteTransferOwnerControl.svelte';
  import NoteActiveEditorsIndicator from '$lib/components/notes/NoteActiveEditorsIndicator.svelte';
  import type { NoteActiveEditor } from '$lib/components/notes/types';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import NoteSnapshot from '$lib/components/notes/NoteSnapshot.svelte';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let transferOpen = $state(false);
  let transferSaving = $state(false);
  let pendingNewOwner = $state<SimpleUserInfo | undefined>(undefined);
  let titleSaving = $state(false);
  let note: NoteInfo | undefined = $state();
  let users: SimpleUserInfo[] | undefined = $state();
  let readonly = $derived(!note?.is_owner);
  let shareSaving = $state(false);
  let publicAccessSaving = $state(false);
  let title = $state('');
  let sharedWithUsers = $state<SharedUserInfo[]>([]);
  let publicAccess = $state<NoteShareAccess | null>(null);
  let sharedUpdateTimeout: ReturnType<typeof setTimeout> | undefined =
    $state(undefined);
  let publicUpdateTimeout: ReturnType<typeof setTimeout> | undefined =
    $state(undefined);
  let userInfo: UserInfo | undefined = $state(undefined);
  let activeEditors = $state<NoteActiveEditor[]>([]);
  let snapshots: NoteSnapshotInfo[] | undefined = $state();

  let shareableUsers = $derived(
    users?.filter((user) => user.id !== note?.owner.id) ?? []
  );

  $effect(() => {
    if (data.error) {
      if (data.error === 'not_found') {
        toast.error('Snapshot not found');
      } else if (data.error === 'other') {
        toast.error('Failed to load snapshot');
      }

      const url = new URL(window.location.href);
      url.searchParams.delete('error');
      window.history.replaceState({}, '', url);
    }
  });

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
      publicAccess = res.data.public_access ?? null;
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

  $effect(() => {
    data.snapshotsPromise.then((snapshotsData) => {
      snapshots = snapshotsData;
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

  const toSharedUserInfo = (
    shares: { userId: string; access: NoteShareAccess }[]
  ): SharedUserInfo[] =>
    shares.map((share) => {
      const user = shareableUsers.find(
        (candidate) => candidate.id === share.userId
      );

      return {
        id: share.userId,
        name: user?.name ?? '',
        access: share.access
      };
    });

  const onSharedChange = async (
    shares: { userId: string; access: NoteShareAccess }[]
  ) => {
    if (!note || readonly) return;

    shareSaving = true;
    const res = await shareNote({
      body: {
        note_id: note.id,
        shared_with: shares.map((share) => ({
          user_id: share.userId,
          access: share.access
        }))
      }
    });
    shareSaving = false;

    if (res.error) {
      sharedWithUsers = note.shared_with;
      toast.error('Failed to update shared users');
    } else {
      const updated = toSharedUserInfo(shares);
      sharedWithUsers = updated;
      note = { ...note, shared_with: updated };
      toast.success('Shared users updated');
    }
  };

  const handleShareChange = (
    shares: { userId: string; access: NoteShareAccess }[]
  ) => {
    sharedWithUsers = toSharedUserInfo(shares);
    if (sharedUpdateTimeout) clearTimeout(sharedUpdateTimeout);
    sharedUpdateTimeout = setTimeout(() => {
      onSharedChange(shares);
    }, 750);
  };

  const onPublicAccessChange = async (access: NoteShareAccess | null) => {
    if (!note || readonly) return;

    publicAccessSaving = true;
    const res = await shareNotePublic({
      body: { note_id: note.id, public_access: access }
    });
    publicAccessSaving = false;

    if (res.error) {
      publicAccess = note.public_access ?? null;
      toast.error('Failed to update public access');
    } else {
      publicAccess = access;
      note = { ...note, public_access: access };
      toast.success(access ? 'Public access updated' : 'Public access removed');
    }
  };

  const handlePublicAccessChange = (access: NoteShareAccess | null) => {
    publicAccess = access;
    if (publicUpdateTimeout) clearTimeout(publicUpdateTimeout);
    publicUpdateTimeout = setTimeout(() => {
      onPublicAccessChange(access);
    }, 750);
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

  const handleTransferRequest = (userId: string) => {
    pendingNewOwner = shareableUsers.find((user) => user.id === userId);
    transferOpen = true;
  };

  const transferConfirm = async () => {
    if (!note || !pendingNewOwner) return;

    transferSaving = true;
    const res = await transferNote({
      body: { note_id: note.id, new_owner_id: pendingNewOwner.id }
    });
    transferSaving = false;

    if (res.error) {
      if (res.response?.status === 409) {
        return {
          error: `${pendingNewOwner.name} has reached the maximum number of notes.`
        };
      }
      return { error: 'Failed to transfer ownership.' };
    }
    pendingNewOwner = undefined;
    toast.success('Ownership transferred');
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mb-0 ml-7 flex min-w-0 items-center gap-2 md:m-0">
    <Button
      size="icon"
      variant="ghost"
      href="/notes"
      class="hidden shrink-0 md:flex"
    >
      <ArrowLeft class="size-5" />
    </Button>

    <Input
      class="bg-background! mr-auto w-full min-w-0 flex-1 border-none text-xl! md:max-w-70"
      bind:value={title}
      placeholder="Note title"
      readonly={readonly || titleSaving}
      onblur={saveTitle}
      onkeydown={(event) => {
        if (event.key === 'Enter') {
          event.currentTarget.blur();
        }
      }}
    />

    <NoteActiveEditorsIndicator editors={activeEditors} />

    <NoteShareControl
      noteId={note?.id ?? data.id}
      {shareableUsers}
      selected={sharedWithUsers}
      {publicAccess}
      onShareChange={handleShareChange}
      onPublicAccessChange={handlePublicAccessChange}
      {readonly}
      saving={shareSaving || publicAccessSaving}
    />
    {#if note && !readonly}
      <NoteTransferOwnerControl
        owner={note.owner}
        candidateUsers={shareableUsers}
        onTransfer={handleTransferRequest}
        saving={transferSaving}
      />
    {:else if note}
      <div
        class="flex h-9 shrink-0 cursor-default items-center gap-2 rounded-full text-sm font-medium md:border md:px-1 lg:pr-2.5 lg:pl-1"
        title={`Owner: ${note.owner.name}`}
      >
        <UserAvatar
          userId={note.owner.id}
          username={note.owner.name}
          class="size-6.5 shrink-0"
        />
        <span class="hidden max-w-32 truncate lg:inline">{note.owner.name}</span
        >
        <Lock class="text-muted-foreground hidden size-3.5 shrink-0 lg:block" />
      </div>
    {/if}
    <Separator orientation="vertical" class="hidden h-5 lg:block" />

    {#if snapshots && note?.is_owner}
      <NoteSnapshot
        {snapshots}
        onOpen={(id) => {
          goto(`/notes/${data.id}/${id}`);
        }}
        onRestore={() => {}}
        onDelete={async (id) => {
          let res = await deleteNoteSnapshot({
            body: {
              snapshot_id: id
            }
          });

          if (res.error) {
            toast.error('Failed to delete snapshot');
          } else {
            toast.success('Snapshot deleted');
          }
        }}
      />
    {/if}
    <Button
      class="shrink-0 cursor-pointer px-2 lg:px-2.5"
      onclick={() => (deleteOpen = true)}
      variant="destructive"
      disabled={readonly}
      aria-label="Delete"
    >
      <Trash />
      <span class="hidden lg:inline">Delete</span>
    </Button>
  </div>
  <div class="flex min-h-0 grow flex-col space-y-4">
    {#if note}
      <TipTab
        id={data.id}
        username={userInfo?.name}
        userId={userInfo?.uuid}
        editable={note.can_edit}
        bind:activeEditors
      />
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
<FormDialog
  title="Transfer Ownership"
  description={`Transfer ownership of "${note?.title}" to ${pendingNewOwner?.name}? You will remain an editor but lose owner controls.`}
  confirm="Transfer"
  onsubmit={transferConfirm}
  bind:open={transferOpen}
  bind:isLoading={transferSaving}
  schema={z.object({})}
/>
