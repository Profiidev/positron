<script lang="ts">
  import type { Editor } from '@tiptap/core';
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { TooltipProvider } from '@profidev/pleiades/components/ui/tooltip';
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

  let { editor }: { editor: Editor } = $props();
</script>

<div class="bg-background sticky top-0 z-20 block w-full border-b">
  <TooltipProvider>
    <div class="flex w-full flex-wrap items-center gap-1 p-1">
      <UndoToolbar {editor} />
      <RedoToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <HeadingsToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <BoldToolbar {editor} />
      <ItalicToolbar {editor} />
      <UnderlineToolbar {editor} />
      <StrikeThroughToolbar {editor} />
      <ColorHighlightToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <AlignmentToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <BlockquoteToolbar {editor} />
      <CodeBlockToolbar {editor} />
      <BulletListToolbar {editor} />
      <OrderedListToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <LinkToolbar {editor} />
      <Separator orientation="vertical" class="mx-1 h-7!" />

      <div class="flex-1"></div>

      <SearchAndReplaceToolbar {editor} />
    </div>
  </TooltipProvider>
</div>
