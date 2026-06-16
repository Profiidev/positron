<script lang="ts">
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import * as Command from '@profidev/pleiades/components/ui/command';
  import { cn } from '@profidev/pleiades/utils';
  import LockOpen from '@lucide/svelte/icons/lock-open';
  import type { SimpleUserInfo } from '$lib/client';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';

  let {
    owner,
    candidateUsers,
    onTransfer,
    saving = false
  }: {
    owner: SimpleUserInfo;
    candidateUsers: SimpleUserInfo[];
    onTransfer: (userId: string) => void;
    saving?: boolean;
  } = $props();

  let open = $state(false);

  const selectUser = (userId: string) => {
    onTransfer(userId);
    open = false;
  };
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class={cn(
      'flex h-9 shrink-0 cursor-pointer items-center gap-2 rounded-full text-sm font-medium transition-colors md:border md:px-1 lg:pr-2.5 lg:pl-1',
      'hover:bg-muted',
      open && 'bg-muted',
      saving && 'pointer-events-none opacity-60'
    )}
    disabled={saving}
    title={`Owner: ${owner.name}`}
    aria-label={`Transfer ownership from ${owner.name}`}
  >
    <UserAvatar
      userId={owner.id}
      username={owner.name}
      class="size-6.5 shrink-0"
    />
    <span class="hidden max-w-32 truncate lg:inline">{owner.name}</span>
    <LockOpen class="text-muted-foreground hidden size-3.5 shrink-0 lg:block" />
  </Popover.Trigger>
  <Popover.Content class="p-0">
    <Command.Root>
      <Command.Input placeholder="Search people..." />
      <Command.List class="flex overflow-hidden">
        <ScrollArea class="mt-1 grow">
          <Command.Empty>No people found</Command.Empty>
          {#each candidateUsers as user (user.id)}
            <Command.Item
              value={user.name}
              class="gap-2"
              onSelect={() => selectUser(user.id)}
            >
              <UserAvatar
                userId={user.id}
                username={user.name}
                class="size-6.5"
              />
              <span class="min-w-0 flex-1 truncate font-medium">
                {user.name}
              </span>
            </Command.Item>
          {/each}
        </ScrollArea>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
