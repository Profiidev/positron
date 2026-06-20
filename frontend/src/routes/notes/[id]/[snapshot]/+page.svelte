<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import ArchiveRestore from '@lucide/svelte/icons/archive-restore';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import {
    deleteNoteSnapshot,
    getNoteSnapshotContent,
    restoreNoteSnapshot,
    type NoteSnapshotDetail
  } from '$lib/client';
  import TipTabReadonly from '$lib/components/tiptap/TipTabReadonly.svelte';
  import { DateTime as D } from '@profidev/pleiades/util/time.svelte';

  const { data } = $props();

  let isLoading = $state(false);
  let restoreOpen = $state(false);
  let deleteSnapshotOpen = $state(false);
  let snapshot: NoteSnapshotDetail | undefined = $state();
  let snapData: Uint8Array | undefined = $state();

  $effect(() => {
    data.snapshotRes.then((res) => {
      if (!res.data) {
        if (res.response?.status === 404) {
          goto(`/notes/${data.id}?error=not_found`);
        } else {
          goto(`/notes/${data.id}?error=other`);
        }
        return;
      }

      snapshot = res.data;
    });
  });

  $effect(() => {
    getNoteSnapshotContent({
      path: {
        snapshot_id: data.snapshot
      },
      parseAs: 'arrayBuffer'
    }).then((res) => {
      if (!res.data || res.error) {
        toast.error(`Failed to load snapshot content`);
        return;
      }
      snapData = new Uint8Array(res.data as ArrayBuffer);
    });
  });

  const deleteSnapshot = async () => {
    isLoading = true;
    let ret = await deleteNoteSnapshot({
      body: {
        snapshot_id: data.snapshot
      }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete snapshot' };
    } else {
      toast.success(`Snapshot deleted successfully`);
      setTimeout(() => {
        goto(`/notes/${data.id}`);
      });
    }
  };

  const restoreConfirm = async () => {
    isLoading = true;
    const res = await restoreNoteSnapshot({
      body: {
        snapshot_id: data.snapshot
      }
    });
    isLoading = false;

    if (res.error) {
      return { error: 'Failed to restore snapshot' };
    } else {
      toast.success(`Snapshot restored successfully`);
      setTimeout(() => {
        goto(`/notes/${data.id}`);
      });
    }
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mb-0 ml-7 flex min-w-0 items-center gap-2 md:m-0">
    <Button
      size="icon"
      variant="ghost"
      href={`/notes/${data.id}`}
      class="shrink-0"
    >
      <ArrowLeft class="size-5" />
    </Button>

    <p class="min-w-0 shrink overflow-hidden text-xl text-nowrap text-ellipsis">
      {snapshot?.title}:
    </p>
    <p class="min-w-0 shrink overflow-hidden text-xl text-nowrap text-ellipsis">
      {snapshot
        ? D.DateTime?.fromISO(snapshot.created_at).toLocaleString(
            D.DateTime.DATETIME_MED_WITH_WEEKDAY,
            {
              locale: navigator.language
            }
          )
        : null}
    </p>

    <Button
      class="ml-auto shrink-0 cursor-pointer px-2 lg:px-2.5"
      onclick={() => (restoreOpen = true)}
      disabled={isLoading}
      aria-label="Restore"
    >
      <ArchiveRestore />
      <span class="hidden lg:inline">Restore</span>
    </Button>
    <Button
      class="shrink-0 cursor-pointer px-2 lg:px-2.5"
      onclick={() => (deleteSnapshotOpen = true)}
      variant="destructive"
      disabled={isLoading}
      aria-label="Delete"
    >
      <Trash />
      <span class="hidden lg:inline">Delete</span>
    </Button>
  </div>
  <div class="flex min-h-0 grow flex-col space-y-4">
    <TipTabReadonly data={snapData ?? new Uint8Array()} />
  </div>
</div>
<FormDialog
  title="Delete Snapshot"
  description={`Do you really want to delete the snapshot ${snapshot?.title}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteSnapshot}
  bind:open={deleteSnapshotOpen}
  bind:isLoading
  schema={z.object({})}
/>
<FormDialog
  title="Restore Snapshot"
  description={`Do you really want to restore this snapshot of ${snapshot?.title}?`}
  confirm="Restore"
  onsubmit={restoreConfirm}
  bind:open={restoreOpen}
  bind:isLoading
  schema={z.object({})}
/>
