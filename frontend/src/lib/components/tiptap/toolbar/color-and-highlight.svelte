<script lang="ts">
  import CheckIcon from '@lucide/svelte/icons/check';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    Popover,
    PopoverContent,
    PopoverTrigger
  } from '@profidev/pleiades/components/ui/popover';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { Separator } from '@profidev/pleiades/components/ui/separator';
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

  const TEXT_COLORS = [
    { name: 'Default', color: 'var(--editor-text-default)' },
    { name: 'Gray', color: 'var(--editor-text-gray)' },
    { name: 'Brown', color: 'var(--editor-text-brown)' },
    { name: 'Orange', color: 'var(--editor-text-orange)' },
    { name: 'Yellow', color: 'var(--editor-text-yellow)' },
    { name: 'Green', color: 'var(--editor-text-green)' },
    { name: 'Blue', color: 'var(--editor-text-blue)' },
    { name: 'Purple', color: 'var(--editor-text-purple)' },
    { name: 'Pink', color: 'var(--editor-text-pink)' },
    { name: 'Red', color: 'var(--editor-text-red)' }
  ] as const;

  const HIGHLIGHT_COLORS = [
    { name: 'Default', color: 'var(--editor-highlight-default)' },
    { name: 'Gray', color: 'var(--editor-highlight-gray)' },
    { name: 'Brown', color: 'var(--editor-highlight-brown)' },
    { name: 'Orange', color: 'var(--editor-highlight-orange)' },
    { name: 'Yellow', color: 'var(--editor-highlight-yellow)' },
    { name: 'Green', color: 'var(--editor-highlight-green)' },
    { name: 'Blue', color: 'var(--editor-highlight-blue)' },
    { name: 'Purple', color: 'var(--editor-highlight-purple)' },
    { name: 'Pink', color: 'var(--editor-highlight-pink)' },
    { name: 'Red', color: 'var(--editor-highlight-red)' }
  ] as const;

  let { editor }: { editor: Editor } = $props();

  const isMobile = new IsMobile();

  const currentColor = $derived(
    editor.getAttributes('textStyle').color as string | undefined
  );
  const currentHighlight = $derived(
    editor.getAttributes('highlight').color as string | undefined
  );
  const isDisabled = $derived(
    !editor.can().chain().setHighlight().run() ||
      !editor.can().chain().setColor('').run()
  );

  function handleSetColor(color: string) {
    editor
      .chain()
      .focus()
      .setColor(color === currentColor ? '' : color)
      .run();
  }

  function handleSetHighlight(color: string) {
    editor
      .chain()
      .focus()
      .setHighlight(color === currentHighlight ? { color: '' } : { color })
      .run();
  }
</script>

{#if isMobile.current}
  <div class="flex gap-1">
    <MobileToolbarGroup label="Color">
      {#snippet children({ closeDrawer })}
        {#each TEXT_COLORS as { name, color } (name)}
          <MobileToolbarItem
            {closeDrawer}
            onclick={() => handleSetColor(color)}
            active={currentColor === color}
          >
            <div class="flex items-center gap-2">
              <div class="rounded-sm border px-2" style:color>A</div>
              <span>{name}</span>
            </div>
          </MobileToolbarItem>
        {/each}
      {/snippet}
    </MobileToolbarGroup>

    <MobileToolbarGroup label="Highlight">
      {#snippet children({ closeDrawer })}
        {#each HIGHLIGHT_COLORS as { name, color } (name)}
          <MobileToolbarItem
            {closeDrawer}
            onclick={() => handleSetHighlight(color)}
            active={currentHighlight === color}
          >
            <div class="flex items-center gap-2">
              <div
                class="rounded-sm border px-2"
                style:background-color={color}
              >
                A
              </div>
              <span>{name}</span>
            </div>
          </MobileToolbarItem>
        {/each}
      {/snippet}
    </MobileToolbarGroup>
  </div>
{:else}
  <Popover>
    <div class="relative h-full">
      <Tooltip>
        <TooltipTrigger>
          {#snippet child({ props })}
            <PopoverTrigger disabled={isDisabled}>
              {#snippet child({ props: triggerProps })}
                <Button
                  {...props}
                  {...triggerProps}
                  variant="ghost"
                  size="sm"
                  type="button"
                  style={currentColor ? `color: ${currentColor}` : undefined}
                  class={cn('h-8 w-14 p-0 font-normal')}
                >
                  <span class="text-md">A</span>
                  <ChevronDownIcon class="ml-2 h-4 w-4" />
                </Button>
              {/snippet}
            </PopoverTrigger>
          {/snippet}
        </TooltipTrigger>
        <TooltipContent>Text Color & Highlight</TooltipContent>
      </Tooltip>

      <PopoverContent align="start" class="dark:bg-gray-2 w-56 p-1">
        <ScrollArea class="max-h-80 overflow-y-auto pr-2">
          <div class="text-gray-11 mt-2 mb-2.5 px-2 text-xs">Color</div>
          {#each TEXT_COLORS as { name, color } (name)}
            <button
              type="button"
              onclick={() => handleSetColor(color)}
              class="hover:bg-gray-3 flex w-full items-center justify-between rounded-sm px-2 py-1 text-sm"
            >
              <div class="flex items-center space-x-2">
                <div
                  class="rounded-sm border px-1 py-px font-medium"
                  style:color
                >
                  A
                </div>
                <span>{name}</span>
              </div>
              {#if currentColor === color}
                <CheckIcon class="h-4 w-4" />
              {/if}
            </button>
          {/each}

          <Separator class="my-3" />

          <div class="text-gray-11 mb-2.5 w-full px-2 pr-3 text-xs">
            Background
          </div>
          {#each HIGHLIGHT_COLORS as { name, color } (name)}
            <button
              type="button"
              onclick={() => handleSetHighlight(color)}
              class="hover:bg-gray-3 flex w-full items-center justify-between rounded-sm px-2 py-1 text-sm"
            >
              <div class="flex items-center space-x-2">
                <div
                  class="rounded-sm border px-1 py-px font-medium"
                  style:background-color={color}
                >
                  A
                </div>
                <span>{name}</span>
              </div>
              {#if currentHighlight === color}
                <CheckIcon class="h-4 w-4" />
              {/if}
            </button>
          {/each}
        </ScrollArea>
      </PopoverContent>
    </div>
  </Popover>
{/if}
