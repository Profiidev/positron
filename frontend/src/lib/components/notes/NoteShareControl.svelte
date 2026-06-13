<script lang="ts">
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import * as Command from '@profidev/pleiades/components/ui/command';
  import { cn } from '@profidev/pleiades/utils';
  import Users from '@lucide/svelte/icons/users';
  import Check from '@lucide/svelte/icons/check';
  import type { SimpleUserInfo } from '$lib/client';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';

  let {
    shareableUsers,
    selected,
    onSelectChange,
    readonly = false,
    saving = false
  }: {
    shareableUsers: SimpleUserInfo[];
    selected: SimpleUserInfo[];
    onSelectChange: (selected: string[]) => void;
    readonly?: boolean;
    saving?: boolean;
  } = $props();

  let open = $state(false);
  let selectedIds = $derived(selected.map((user) => user.id));

  const extraCount = $derived(Math.max(0, selected.length - 4));

  const toggleUser = (id: string) => {
    if (selectedIds.includes(id)) {
      selectedIds = selectedIds.filter((userId) => userId !== id);
    } else {
      selectedIds = [...selectedIds, id];
    }
    onSelectChange(selectedIds);
  };
</script>

{#if readonly}
  <div
    class="flex h-9 shrink-0 cursor-default items-center gap-2 rounded-full border px-1.5 text-sm font-medium md:px-3.5 md:pl-1.5"
    title="Shared with"
  >
    {#if selected.length > 0}
      <div
        class="*:data-[slot=avatar]:ring-background flex -space-x-2 *:data-[slot=avatar]:ring-2"
      >
        {#each selected.slice(0, 4) as user (user.id)}
          <UserAvatar userId={user.id} username={user.name} class="size-6" />
        {/each}
      </div>
      <span class="hidden md:inline">{selected.length} shared</span>
    {:else}
      <Users class="text-muted-foreground mx-1 size-4 md:mx-0" />
      <span class="text-muted-foreground hidden md:inline">Share</span>
    {/if}
  </div>
{:else}
  <Popover.Root bind:open>
    <Popover.Trigger
      class={cn(
        'flex h-9 shrink-0 cursor-pointer items-center gap-2 rounded-full border px-1.5 text-sm font-medium transition-colors md:px-3.5 md:pl-1.5',
        'hover:bg-muted',
        open && 'bg-muted',
        saving && 'pointer-events-none opacity-60'
      )}
      disabled={saving}
    >
      {#if selected.length > 0}
        <div
          class="*:data-[slot=avatar]:ring-background flex -space-x-2 *:data-[slot=avatar]:ring-2"
        >
          {#each selected.slice(0, 4) as user (user.id)}
            <UserAvatar userId={user.id} username={user.name} class="size-6" />
          {/each}
          {#if extraCount > 0}
            <div
              class="bg-muted text-muted-foreground ring-background z-10 flex size-6 items-center justify-center rounded-full text-[10px] font-semibold ring-2"
            >
              +{extraCount}
            </div>
          {/if}
        </div>
        <span class="hidden md:inline">{selected.length} shared</span>
      {:else}
        <Users class="text-muted-foreground mx-1 size-4 md:mr-0 md:ml-2" />
        <span class="text-muted-foreground hidden md:inline">Share</span>
      {/if}
    </Popover.Trigger>
    <Popover.Content class="p-0">
      <Command.Root>
        <Command.Input placeholder="Search people..." />
        <Command.List class="flex overflow-hidden">
          <ScrollArea class="mt-1 grow">
            <Command.Empty>No people found</Command.Empty>
            {#each shareableUsers as user (user.id)}
              <Command.Item
                value={user.name}
                onSelect={() => toggleUser(user.id)}
                class="[&_svg.cn-command-item-indicator]:hidden!"
              >
                <UserAvatar
                  userId={user.id}
                  username={user.name}
                  class="size-6.5"
                />
                <span class="min-w-0 flex-1 truncate font-medium">
                  {user.name}
                </span>
                <Check
                  class={cn(
                    'mr-2 ml-auto size-4',
                    !selectedIds.includes(user.id) && 'text-transparent!'
                  )}
                />
              </Command.Item>
            {/each}
          </ScrollArea>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Root>
{/if}
