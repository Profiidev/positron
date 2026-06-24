<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import ArchiveRestore from '@lucide/svelte/icons/archive-restore';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import {
    deleteNoteSnapshot,
    noteSnapshotContent,
    noteSnapshotInfo,
    restoreNoteSnapshot,
    type NoteSnapshotDetail
  } from '$lib/commands/notes.svelte';
  import Nav from '$lib/components/Nav.svelte';
  import TipTabReadonly from '$lib/components/tiptap/TipTabReadonly.svelte';
  import { DateTime as D } from '@profidev/pleiades/util/time.svelte';
  import { onMount } from 'svelte';
  import { onNotesUpdate } from '$lib/updater/state.svelte';

  const id = $derived(page.params.id ?? '');
  const snapshotId = $derived(page.params.snapshot ?? '');

  let isLoading = $state(false);
  let restoreOpen = $state(false);
  let deleteSnapshotOpen = $state(false);
  let snapshot: NoteSnapshotDetail | undefined = $state();
  let snapData: Uint8Array | undefined = $state();

  const loadData = () => {
    noteSnapshotInfo(snapshotId).then((res) => {
      if (!res) {
        goto(`/notes/${id}?error=not_found`);
        return;
      }
      snapshot = res;
    });
  };

  onMount(onNotesUpdate(loadData));

  $effect(() => {
    snapshotId;
    loadData();
  });

  $effect(() => {
    noteSnapshotContent(snapshotId).then((res) => {
      if (!res) {
        toast.error('Failed to load snapshot content');
        return;
      }
      snapData = res;
    });
  });

  const deleteSnapshot = async () => {
    isLoading = true;
    const ok = await deleteNoteSnapshot(snapshotId);
    isLoading = false;

    if (!ok) {
      return { error: 'Failed to delete snapshot' };
    } else {
      toast.success(`Snapshot deleted successfully`);
      setTimeout(() => {
        goto(`/notes/${id}`);
      });
    }
  };

  const restoreConfirm = async () => {
    isLoading = true;
    const ok = await restoreNoteSnapshot(snapshotId);
    isLoading = false;

    if (!ok) {
      return { error: 'Failed to restore snapshot' };
    } else {
      toast.success(`Snapshot restored successfully`);
      setTimeout(() => {
        goto(`/notes/${id}`);
      });
    }
  };
</script>

<div class="flex min-h-0 w-full flex-1 flex-col space-y-6 p-4 pt-1">
  <div class="mb-0 flex min-w-0 items-center gap-2">
    <Button size="icon" variant="ghost" href={`/notes/${id}`} class="shrink-0">
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
