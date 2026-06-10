<script lang="ts">
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger
  } from '@profidev/pleiades/components/ui/dropdown-menu';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
  import { IsMobile } from '@profidev/pleiades/hooks/is-mobile.svelte';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import MobileToolbarGroup from './mobile-toolbar-group.svelte';
  import MobileToolbarItem from './MobileToolbarItem.svelte';

  const levels = [1, 2, 3, 4] as const;

  let { editor, class: className }: { editor: Editor; class?: string } =
    $props();

  const isMobile = new IsMobile();

  const activeLevel = $derived(
    levels.find((level) => editor.isActive('heading', { level }))
  );
  const isHeadingActive = $derived(editor.isActive('heading'));
</script>

{#if isMobile.current}
  <MobileToolbarGroup label={activeLevel ? `H${activeLevel}` : 'Normal'}>
    {#snippet children({ closeDrawer })}
      <MobileToolbarItem
        {closeDrawer}
        onclick={() => editor.chain().focus().setParagraph().run()}
        active={!editor.isActive('heading')}
      >
        Normal
      </MobileToolbarItem>
      {#each levels as level (level)}
        <MobileToolbarItem
          {closeDrawer}
          onclick={() => editor.chain().focus().toggleHeading({ level }).run()}
          active={editor.isActive('heading', { level })}
        >
          H{level}
        </MobileToolbarItem>
      {/each}
    {/snippet}
  </MobileToolbarGroup>
{:else}
  <Tooltip>
    <TooltipTrigger>
      {#snippet child({ props })}
        <DropdownMenu>
          <DropdownMenuTrigger>
            {#snippet child({ props: triggerProps })}
              <Button
                {...props}
                {...triggerProps}
                variant="ghost"
                size="sm"
                type="button"
                class={cn(
                  'h-8 w-max cursor-pointer gap-1 px-3 font-normal',
                  className
                )}
              >
                {activeLevel ? `H${activeLevel}` : 'Normal'}
                <ChevronDownIcon class="h-4 w-4" />
              </Button>
            {/snippet}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" class="flex flex-col gap-1">
            <DropdownMenuItem
              onclick={() => editor.chain().focus().setParagraph().run()}
              class={cn(
                'flex h-fit items-center gap-2',
                !isHeadingActive && 'bg-accent'
              )}
            >
              Normal
            </DropdownMenuItem>
            {#each levels as level (level)}
              <DropdownMenuItem
                onclick={() =>
                  editor.chain().focus().toggleHeading({ level }).run()}
                class={cn(
                  'flex items-center gap-2',
                  editor.isActive('heading', { level }) && 'bg-accent'
                )}
              >
                H{level}
              </DropdownMenuItem>
            {/each}
          </DropdownMenuContent>
        </DropdownMenu>
      {/snippet}
    </TooltipTrigger>
    <TooltipContent>
      <span>Headings</span>
    </TooltipContent>
  </Tooltip>
{/if}
