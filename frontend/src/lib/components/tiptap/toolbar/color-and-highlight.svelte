<script lang="ts">
  import CheckIcon from '@lucide/svelte/icons/check';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import PaletteIcon from '@lucide/svelte/icons/palette';
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
  import * as Command from '@profidev/pleiades/components/ui/command';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

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

  let {
    editor,
    inOverflowMenu = false
  }: {
    editor: Editor;
    inOverflowMenu?: boolean;
  } = $props();

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

  const handleSetColor = (color: string) => {
    editor
      .chain()
      .focus()
      .setColor(color === currentColor ? '' : color)
      .run();
  };

  const handleSetHighlight = (color: string) => {
    editor
      .chain()
      .focus()
      .setHighlight(color === currentHighlight ? { color: '' } : { color })
      .run();
  };
</script>

{#snippet colorMenu()}
  <PopoverContent class="w-52 p-0">
    <Command.Root>
      <Command.List class="flex overflow-hidden">
        <ScrollArea class="grow">
          <Command.Group heading="Color">
            {#each TEXT_COLORS as { name, color } (name)}
              <Command.Item
                onSelect={() => handleSetColor(color)}
                class="flex w-full cursor-pointer items-center rounded-sm px-2 py-1 text-sm [&_svg.cn-command-item-indicator]:hidden!"
              >
                <div
                  class="rounded-sm border px-1 py-px font-medium"
                  style:color
                >
                  A
                </div>
                <span>{name}</span>
                {#if currentColor === color}
                  <CheckIcon class="ml-auto h-4 w-4" />
                {/if}
              </Command.Item>
            {/each}
          </Command.Group>

          <Separator class="my-1" />

          <Command.Group heading="Background">
            {#each HIGHLIGHT_COLORS as { name, color } (name)}
              <Command.Item
                onSelect={() => handleSetHighlight(color)}
                class="flex w-full cursor-pointer items-center rounded-sm px-2 py-1 text-sm [&_svg.cn-command-item-indicator]:hidden!"
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
                  <CheckIcon class="ml-auto h-4 w-4" />
                {/if}
              </Command.Item>
            {/each}
          </Command.Group>
        </ScrollArea>
      </Command.List>
    </Command.Root>
  </PopoverContent>
{/snippet}

<Popover>
  {#if inOverflowMenu}
    <PopoverTrigger disabled={isDisabled}>
      {#snippet child({ props })}
        <ToolbarOverflowTrigger
          {...props}
          label="Color & Highlight"
          icon={PaletteIcon}
          hasSubmenu
          disabled={isDisabled}
        />
      {/snippet}
    </PopoverTrigger>
    {@render colorMenu()}
  {:else}
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
                  class={cn('h-8 w-14 cursor-pointer p-0 font-normal')}
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
      {@render colorMenu()}
    </div>
  {/if}
</Popover>
