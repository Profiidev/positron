<script lang="ts">
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import * as Command from '@profidev/pleiades/components/ui/command';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import History from '@lucide/svelte/icons/history';
  import ArchiveRestore from '@lucide/svelte/icons/archive-restore';
  import type { NoteSnapshotInfo } from './types';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { DateTime as D } from '@profidev/pleiades/util/time.svelte';
  import {
    Button,
    buttonVariants
  } from '@profidev/pleiades/components/ui/button';
  import Trash from '@lucide/svelte/icons/trash';

  let {
    onOpen,
    onRestore,
    onDelete,
    snapshots,
    saving = false
  }: {
    onOpen: (snapshotId: string) => void;
    onRestore: (snapshotId: string) => void;
    onDelete: (snapshotId: string) => void;
    snapshots: NoteSnapshotInfo[];
    saving?: boolean;
  } = $props();

  let open = $state(false);

  const openSnapshot = (snapshotId: string) => {
    open = false;
    onOpen(snapshotId);
  };

  const restoreSnapshot = (snapshotId: string) => {
    open = false;
    onRestore(snapshotId);
  };

  const deleteSnapshot = (snapshotId: string) => {
    open = false;
    onDelete(snapshotId);
  };
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class={cn(
      buttonVariants({ variant: 'outline' }),
      'flex cursor-pointer items-center px-1.5 lg:px-2.5',
      open && 'bg-muted',
      saving && 'pointer-events-none opacity-60'
    )}
    disabled={saving}
    title={`Snapshot manager`}
    aria-label="Snapshot manager"
  >
    <History class="size-5" />
    <span class="hidden lg:inline">Snapshots</span>
  </Popover.Trigger>
  <Popover.Content class="p-0">
    <Command.Root>
      <Command.Input placeholder="Search new owner..." />
      <Command.List class="flex overflow-hidden">
        <ScrollArea class="mt-2 grow">
          <Command.Empty>No people found</Command.Empty>
          {#each snapshots as snapshot (snapshot.id)}
            {let created_at = $derived(
              D.DateTime?.fromISO(snapshot.created_at).toLocaleString(
                D.DateTime.DATETIME_MED_WITH_WEEKDAY,
                {
                  locale: navigator.language
                }
              )
            )}

            <Tooltip.Provider delayDuration={300}>
              <Tooltip.Root>
                <Tooltip.Trigger>
                  {#snippet child({ props })}
                    <Command.Item
                      value={created_at}
                      class="cursor-pointer gap-2 [&_svg.cn-command-item-indicator]:hidden!"
                      onSelect={() => openSnapshot(snapshot.id)}
                      {...props}
                    >
                      <span class="min-w-0 flex-1 truncate font-medium">
                        {created_at}
                      </span>
                      <Tooltip.Root>
                        <Tooltip.Trigger
                          class={buttonVariants({
                            variant: 'outline',
                            size: 'icon',
                            class: 'cursor-pointer'
                          })}
                          onclick={(e) => {
                            e.stopPropagation();
                            restoreSnapshot(snapshot.id);
                          }}
                        >
                          <ArchiveRestore />
                        </Tooltip.Trigger>
                        <Tooltip.Content>
                          <p>Restore Snapshot</p>
                        </Tooltip.Content>
                      </Tooltip.Root>
                      <Button
                        size="icon"
                        class="cursor-pointer"
                        variant="destructive"
                        onclick={(e) => {
                          e.stopPropagation();
                          deleteSnapshot(snapshot.id);
                        }}
                      >
                        <Trash />
                      </Button>
                    </Command.Item>
                  {/snippet}
                </Tooltip.Trigger>
                <Tooltip.Content>
                  <p>View Snapshot</p>
                </Tooltip.Content>
              </Tooltip.Root>
            </Tooltip.Provider>
          {/each}
        </ScrollArea>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
