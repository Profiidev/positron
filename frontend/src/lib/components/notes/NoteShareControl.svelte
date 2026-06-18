<script lang="ts">
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import * as Command from '@profidev/pleiades/components/ui/command';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { CopyButton } from '@profidev/pleiades/components/ui-extra/copy-button';
  import { cn } from '@profidev/pleiades/utils';
  import Users from '@lucide/svelte/icons/users';
  import Globe from '@lucide/svelte/icons/globe';
  import Eye from '@lucide/svelte/icons/eye';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Link from '@lucide/svelte/icons/link';
  import type {
    NoteShareAccess,
    SharedUserInfo,
    SimpleUserInfo
  } from '$lib/client';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { page } from '$app/state';

  type ShareEntry = { userId: string; access: NoteShareAccess };

  let {
    noteId,
    shareableUsers,
    selected,
    publicAccess = null,
    onShareChange,
    onPublicAccessChange,
    readonly = false,
    saving = false
  }: {
    noteId: string;
    shareableUsers: SimpleUserInfo[];
    selected: SharedUserInfo[];
    publicAccess?: NoteShareAccess | null;
    onShareChange: (shares: ShareEntry[]) => void;
    onPublicAccessChange: (access: NoteShareAccess | null) => void;
    readonly?: boolean;
    saving?: boolean;
  } = $props();

  let open = $state(false);

  const shares = $derived(
    selected.map((user) => ({ userId: user.id, access: user.access }))
  );

  const extraCount = $derived(Math.max(0, selected.length - 4));
  const shareUrl = $derived(`${page.url.origin}/notes/share/${noteId}`);
  const isPublic = $derived(publicAccess !== null);

  const shareForUser = (userId: string) =>
    shares.find((share) => share.userId === userId);

  const setAccess = (userId: string, access: NoteShareAccess) => {
    const current = shareForUser(userId);
    if (!current) {
      onShareChange([...shares, { userId, access }]);
      return;
    }
    if (current.access === access) {
      onShareChange(shares.filter((share) => share.userId !== userId));
      return;
    }
    onShareChange(
      shares.map((share) =>
        share.userId === userId ? { userId, access } : share
      )
    );
  };

  const setPublicAccess = (access: NoteShareAccess) => {
    if (publicAccess === access) {
      onPublicAccessChange(null);
      return;
    }
    onPublicAccessChange(access);
  };
</script>

{#if readonly}
  <div
    class="flex h-9 shrink-0 cursor-default items-center gap-2 rounded-full border px-1.5 text-sm font-medium md:px-3.5 md:pl-1.5"
    title="Shared with"
  >
    {#if selected.length > 0 || isPublic}
      <div
        class="*:data-[slot=avatar]:ring-background flex -space-x-2 *:data-[slot=avatar]:ring-2"
      >
        {#if isPublic}
          <div
            class="bg-muted text-muted-foreground ring-background flex size-6 items-center justify-center rounded-full ring-2"
          >
            <Globe class="size-3.5" />
          </div>
        {/if}
        {#each selected.slice(0, isPublic ? 3 : 4) as user (user.id)}
          <UserAvatar userId={user.id} username={user.name} class="size-6" />
        {/each}
      </div>
      <span class="hidden md:inline">
        {#if isPublic && selected.length === 0}
          Public
        {:else}
          {selected.length + (isPublic ? 1 : 0)} shared
        {/if}
      </span>
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
      {#if selected.length > 0 || isPublic}
        <div
          class="*:data-[slot=avatar]:ring-background flex -space-x-2 *:data-[slot=avatar]:ring-2"
        >
          {#if isPublic}
            <div
              class="bg-muted text-muted-foreground ring-background flex size-6 items-center justify-center rounded-full ring-2"
            >
              <Globe class="size-3.5" />
            </div>
          {/if}
          {#each selected.slice(0, isPublic ? 3 : 4) as user (user.id)}
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
        <span class="hidden md:inline">
          {#if isPublic && selected.length === 0}
            Public
          {:else}
            {selected.length + (isPublic ? 1 : 0)} shared
          {/if}
        </span>
      {:else}
        <Users class="text-muted-foreground mx-1 size-4 md:mr-0 md:ml-2" />
        <span class="text-muted-foreground hidden md:inline">Share</span>
      {/if}
    </Popover.Trigger>
    <Popover.Content class="w-80 gap-0 p-0">
      <div class="flex flex-col gap-2 border-b p-2">
        <div class="flex items-center gap-2 px-1">
          <div
            class="bg-muted text-muted-foreground flex size-6.5 shrink-0 items-center justify-center rounded-full"
          >
            <Globe class="size-3.5" />
          </div>
          <span class="min-w-0 flex-1 truncate font-medium">Public access</span>
          <div
            class="bg-muted flex shrink-0 rounded-md border p-0.5"
            role="group"
            aria-label="Public share access"
          >
            <Button
              type="button"
              size="sm"
              variant={publicAccess === 'view' ? 'default' : 'ghost'}
              onclick={() => setPublicAccess('view')}
            >
              <Eye class="size-3.5" />
              View
            </Button>
            <Button
              type="button"
              size="sm"
              variant={publicAccess === 'edit' ? 'default' : 'ghost'}
              onclick={() => setPublicAccess('edit')}
            >
              <Pencil class="size-3.5" />
              Edit
            </Button>
          </div>
        </div>
        {#if isPublic}
          <CopyButton
            text={shareUrl}
            variant="outline"
            class="h-8 w-full justify-start gap-2"
          >
            <Link class="size-3.5 shrink-0" />
            <span class="truncate">Copy share link</span>
          </CopyButton>
        {/if}
      </div>
      <Command.Root>
        <Command.Input placeholder="Search people..." />
        <Command.List class="flex overflow-hidden pt-1">
          <ScrollArea class="mt-1 grow">
            <Command.Empty>No people found</Command.Empty>
            {#each shareableUsers as user (user.id)}
              {@const share = shareForUser(user.id)}
              <Command.Item
                value={user.name}
                class="gap-2 [&_svg.cn-command-item-indicator]:hidden!"
              >
                <UserAvatar
                  userId={user.id}
                  username={user.name}
                  class="size-6.5"
                />
                <span class="min-w-0 flex-1 truncate font-medium">
                  {user.name}
                </span>
                <div
                  class="bg-muted flex shrink-0 rounded-md border p-0.5"
                  role="group"
                  aria-label="Share access"
                >
                  <Button
                    type="button"
                    size="sm"
                    variant={share?.access === 'view' ? 'default' : 'ghost'}
                    onclick={(event) => {
                      event.stopPropagation();
                      setAccess(user.id, 'view');
                    }}
                  >
                    <Eye class="size-3.5" />
                    View
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant={share?.access === 'edit' ? 'default' : 'ghost'}
                    onclick={(event) => {
                      event.stopPropagation();
                      setAccess(user.id, 'edit');
                    }}
                  >
                    <Pencil class="size-3.5" />
                    Edit
                  </Button>
                </div>
              </Command.Item>
            {/each}
          </ScrollArea>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Root>
{/if}
