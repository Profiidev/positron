<script lang="ts">
  import { BubbleMenu, type Editor } from 'svelte-tiptap';
  import { IsMobile } from '@profidev/pleiades/hooks/is-mobile.svelte';
  import {
    ScrollArea,
    ScrollAreaScrollbar
  } from '@profidev/pleiades/components/ui/scroll-area';
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { TooltipProvider } from '@profidev/pleiades/components/ui/tooltip';
  import BoldToolbar from '../toolbar/bold.svelte';
  import ItalicToolbar from '../toolbar/italic.svelte';
  import UnderlineToolbar from '../toolbar/underline.svelte';
  import LinkToolbar from '../toolbar/link.svelte';
  import ColorHighlightToolbar from '../toolbar/color-and-highlight.svelte';
  import HeadingsToolbar from '../toolbar/headings.svelte';
  import BulletListToolbar from '../toolbar/bullet-list.svelte';
  import OrderedListToolbar from '../toolbar/ordered-list.svelte';
  import AlignmentToolbar from '../toolbar/alignment.svelte';
  import BlockquoteToolbar from '../toolbar/blockquote.svelte';

  let { editor }: { editor: Editor } = $props();

  const isMobile = new IsMobile();

  $effect(() => {
    if (!isMobile.current) return;

    let cleanup: (() => void) | undefined;

    const attach = () => {
      const el = editor.view?.dom;
      if (!el) return;

      const handleContextMenu = (e: Event) => {
        e.preventDefault();
      };

      el.addEventListener('contextmenu', handleContextMenu);
      cleanup = () => el.removeEventListener('contextmenu', handleContextMenu);
    };

    if (editor.view) {
      attach();
    } else {
      const onCreate = () => attach();
      editor.on('create', onCreate);
      cleanup = () => editor.off('create', onCreate);
    }

    return () => cleanup?.();
  });
</script>

{#if isMobile.current}
  <TooltipProvider>
    <BubbleMenu
      {editor}
      options={{
        placement: 'bottom',
        offset: 10
      }}
      shouldShow={({ editor: currentEditor }) =>
        currentEditor.isEditable && currentEditor.isFocused}
      class="bg-background mx-0 w-full min-w-full rounded-sm border shadow-sm"
    >
      <ScrollArea class="h-fit w-full py-0.5">
        <div class="flex items-center gap-0.5 px-2">
          <div class="flex items-center gap-0.5 p-1">
            <BoldToolbar {editor} />
            <ItalicToolbar {editor} />
            <UnderlineToolbar {editor} />
            <Separator orientation="vertical" class="mx-1 h-6" />

            <HeadingsToolbar {editor} />
            <BulletListToolbar {editor} />
            <OrderedListToolbar {editor} />
            <Separator orientation="vertical" class="mx-1 h-6" />

            <ColorHighlightToolbar {editor} />
            <LinkToolbar {editor} />
            <Separator orientation="vertical" class="mx-1 h-6" />

            <AlignmentToolbar {editor} />
            <BlockquoteToolbar {editor} />
          </div>
        </div>
        <ScrollAreaScrollbar class="h-0.5" orientation="horizontal" />
      </ScrollArea>
    </BubbleMenu>
  </TooltipProvider>
{/if}
