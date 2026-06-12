<script lang="ts">
  import type { Component } from 'svelte';
  import type { Editor } from '@tiptap/core';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuTrigger
  } from '@profidev/pleiades/components/ui/dropdown-menu';
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
  import UndoToolbar from './undo.svelte';
  import RedoToolbar from './redo.svelte';
  import HeadingsToolbar from './headings.svelte';
  import BlockquoteToolbar from './blockquote.svelte';
  import BoldToolbar from './bold.svelte';
  import ItalicToolbar from './italic.svelte';
  import UnderlineToolbar from './underline.svelte';
  import StrikeThroughToolbar from './strikethrough.svelte';
  import LinkToolbar from './link.svelte';
  import BulletListToolbar from './bullet-list.svelte';
  import OrderedListToolbar from './ordered-list.svelte';
  import AlignmentToolbar from './alignment.svelte';
  import ColorHighlightToolbar from './color-and-highlight.svelte';
  import SearchAndReplaceToolbar from './search-and-replace-toolbar.svelte';
  import CodeBlockToolbar from './code-block.svelte';
  import {
    calculateVisibleCount,
    cleanupSeparatorVisibility,
    type ToolbarOverflowItem
  } from './toolbar-overflow';

  const GAP = 4;

  type ToolbarComponent = Component<{
    editor: Editor;
    inOverflowMenu?: boolean;
    class?: string;
  }>;

  type ToolbarEntry = ToolbarOverflowItem & {
    component?: ToolbarComponent;
  };

  let { editor }: { editor: Editor } = $props();

  const items: ToolbarEntry[] = [
    { id: 'undo', component: UndoToolbar },
    { id: 'redo', component: RedoToolbar },
    { id: 'sep-history', isSeparator: true },
    { id: 'headings', component: HeadingsToolbar },
    { id: 'sep-headings', isSeparator: true },
    { id: 'bold', component: BoldToolbar },
    { id: 'italic', component: ItalicToolbar },
    { id: 'underline', component: UnderlineToolbar },
    { id: 'strikethrough', component: StrikeThroughToolbar },
    { id: 'color', component: ColorHighlightToolbar },
    { id: 'sep-format', isSeparator: true },
    { id: 'alignment', component: AlignmentToolbar },
    { id: 'sep-alignment', isSeparator: true },
    { id: 'blockquote', component: BlockquoteToolbar },
    { id: 'code-block', component: CodeBlockToolbar },
    { id: 'bullet-list', component: BulletListToolbar },
    { id: 'ordered-list', component: OrderedListToolbar },
    { id: 'sep-blocks', isSeparator: true },
    { id: 'link', component: LinkToolbar },
    { id: 'sep-link', isSeparator: true },
    { id: 'search', component: SearchAndReplaceToolbar }
  ];

  let containerWidth = $state(0);
  let overflowButtonWidth = $state(32);
  let itemWidths = $state<number[]>([]);

  const layout = $derived.by(() => {
    const widths =
      itemWidths.length === items.length ? itemWidths : items.map(() => 0);

    const { visibleCount, showOverflow } = calculateVisibleCount(
      containerWidth,
      widths,
      overflowButtonWidth,
      GAP
    );

    const visibility = cleanupSeparatorVisibility(items, visibleCount);

    return { visibleCount, showOverflow, visibility };
  });
</script>

{#snippet toolbarSeparator()}
  <Separator orientation="vertical" class="mx-1 h-7!" />
{/snippet}

{#snippet renderItem(item: ToolbarEntry, inOverflowMenu: boolean)}
  {#if item.isSeparator}
    {#if !inOverflowMenu}
      {@render toolbarSeparator()}
    {/if}
  {:else if item.component}
    <item.component {editor} {inOverflowMenu} />
  {/if}
{/snippet}

<div class="relative w-full">
  <div
    class="flex w-full items-center gap-1 overflow-hidden p-1"
    bind:offsetWidth={containerWidth}
  >
    {#each items as item, index (item.id)}
      {#if index < layout.visibleCount && layout.visibility[index]}
        {@render renderItem(item, false)}
      {/if}
    {/each}

    {#if layout.showOverflow}
      <DropdownMenu>
        <Tooltip>
          <TooltipTrigger>
            {#snippet child({ props })}
              <DropdownMenuTrigger>
                {#snippet child({ props: triggerProps })}
                  <Button
                    {...props}
                    {...triggerProps}
                    variant="ghost"
                    size="icon"
                    type="button"
                    class="ml-auto cursor-pointer"
                    aria-label="More tools"
                  >
                    <MoreHorizontalIcon />
                  </Button>
                {/snippet}
              </DropdownMenuTrigger>
            {/snippet}
          </TooltipTrigger>
          <TooltipContent>More tools</TooltipContent>
        </Tooltip>
        <DropdownMenuContent
          align="end"
          class="w-52 p-1"
          onCloseAutoFocus={(e) => e.preventDefault()}
        >
          {#each items as item, index (item.id)}
            {#if index >= layout.visibleCount && !item.isSeparator}
              {@render renderItem(item, true)}
            {/if}
          {/each}
        </DropdownMenuContent>
      </DropdownMenu>
    {/if}
  </div>

  <!-- used for width calculations -->
  <div
    class="pointer-events-none invisible absolute top-0 left-0 flex gap-1 p-1"
    aria-hidden="true"
  >
    {#each items as item, index (item.id)}
      <div bind:offsetWidth={itemWidths[index]}>
        {@render renderItem(item, false)}
      </div>
    {/each}

    <div bind:offsetWidth={overflowButtonWidth}>
      <Button variant="ghost" size="icon" type="button" tabindex={-1}>
        <MoreHorizontalIcon />
      </Button>
    </div>
  </div>
</div>
