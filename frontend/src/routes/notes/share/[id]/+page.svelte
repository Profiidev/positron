<script lang="ts">
  import Lock from '@lucide/svelte/icons/lock';
  import { type NoteInfoPublic } from '$lib/client';
  import TipTab from '$lib/components/tiptap/TipTab.svelte';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import NoteActiveEditorsIndicator from '$lib/components/notes/NoteActiveEditorsIndicator.svelte';
  import type { NoteActiveEditor } from '$lib/components/notes/types';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { _UNKNOWN_EMAIL } from '$routes/+layout.js';
  import { goto } from '$app/navigation';
  import {
    connectPublicNoteUpdater,
    disconnectPublicNoteUpdater
  } from '$lib/backend/public-note-updater.svelte.js';
  import { onDestroy } from 'svelte';

  const { data } = $props();

  let note: NoteInfoPublic | undefined = $state();
  let title = $state('');
  let userInfo:
    | {
        uuid: string;
        name: string;
      }
    | undefined = $state(undefined);
  let activeEditors = $state<NoteActiveEditor[]>([]);

  $effect(() => {
    data.noteRes.then((res) => {
      if (!res.data) {
        toast.error('Failed to load note');
        goto('/login');
        return;
      }

      note = res.data;
      title = res.data.title;
      connectPublicNoteUpdater(data.id);
    });
  });

  $effect(() => {
    data.user.then((userInfoData) => {
      if (userInfoData.email === _UNKNOWN_EMAIL) {
        userInfo = {
          uuid: crypto.randomUUID(),
          name: 'Anonymous'
        };
      } else {
        goto(`/notes/${data.id}`);
      }
    });
  });

  onDestroy(disconnectPublicNoteUpdater);
</script>

<svelte:window onbeforeunload={disconnectPublicNoteUpdater} />

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mb-0 flex min-w-0 items-center gap-2">
    <Input
      class="bg-background! mr-auto w-full min-w-0 flex-1 border-none text-xl! md:max-w-70"
      bind:value={title}
      placeholder="Note title"
      readonly
    />

    <NoteActiveEditorsIndicator editors={activeEditors} />

    {#if note}
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
  </div>
  <div class="flex min-h-0 grow flex-col space-y-4">
    {#if note}
      <TipTab
        id={data.id}
        username={userInfo?.name}
        userId={userInfo?.uuid}
        editable={note.can_edit}
        bind:activeEditors
        wsPath="/api/notes/websocket/public"
      />
    {/if}
  </div>
</div>
