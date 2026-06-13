<script lang="ts">
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import { cn } from '@profidev/pleiades/utils';
  import UserAvatar from '$lib/components/UserAvatar.svelte';
  import type { NoteActiveEditor } from '$lib/components/notes/types';

  let { editors }: { editors: NoteActiveEditor[] } = $props();

  let open = $state(false);

  const extraCount = $derived(Math.max(0, editors.length - 4));
</script>

{#if editors.length > 0}
  <Popover.Root bind:open>
    <Popover.Trigger
      class={cn(
        'flex h-9 shrink-0 cursor-pointer items-center gap-2 rounded-full border px-3.5 pl-1.5 text-sm font-medium transition-colors',
        'hover:bg-muted',
        open && 'bg-muted'
      )}
      title="Currently editing"
    >
      <div
        class="*:data-[slot=avatar]:ring-background flex -space-x-2 *:data-[slot=avatar]:ring-2"
      >
        {#each editors.slice(0, 4) as editor, index (editor.clientId)}
          <div
            class="rounded-full"
            style:z-index={index}
            style:box-shadow={editor.color
              ? `0 0 0 2px var(--background), 0 0 0 4px ${editor.color}`
              : undefined}
          >
            <UserAvatar
              userId={editor.id}
              username={editor.name}
              class="size-6"
            />
          </div>
        {/each}
        {#if extraCount > 0}
          <div
            class="bg-muted text-muted-foreground ring-background z-10 flex size-6 items-center justify-center rounded-full text-[10px] font-semibold ring-2"
          >
            +{extraCount}
          </div>
        {/if}
      </div>
      <span>{editors.length} editing</span>
    </Popover.Trigger>
    <Popover.Content class="w-[240px] p-2" align="end">
      <p
        class="text-muted-foreground px-1 text-xs font-medium tracking-wide uppercase"
      >
        Currently editing
      </p>
      <ul class="space-y-1">
        {#each editors as editor (editor.clientId)}
          <li class="flex items-center gap-2.5 rounded-md px-1 py-1.5 text-sm">
            <UserAvatar userId={editor.id} username={editor.name} class="size-6" />
            <span class="min-w-0 flex-1 truncate font-medium">{editor.name}</span>
            {#if editor.color}
              <span
                class="size-2.5 shrink-0 rounded-full"
                style:background-color={editor.color}
                title="Caret color"
              ></span>
            {/if}
          </li>
        {/each}
      </ul>
    </Popover.Content>
  </Popover.Root>
{/if}
