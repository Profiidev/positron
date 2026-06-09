<script lang="ts">
  import UnderlineIcon from '@lucide/svelte/icons/underline';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';

  let { editor, class: className }: { editor: Editor; class?: string } =
    $props();

  const isActive = $derived(editor.isActive('underline'));
  const isDisabled = $derived(
    !editor.can().chain().focus().toggleUnderline().run()
  );

  function handleClick() {
    editor.chain().focus().toggleUnderline().run();
  }
</script>

<Tooltip>
  <TooltipTrigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        size="icon"
        type="button"
        class={cn('cursor-pointer', isActive && 'bg-accent', className)}
        onclick={handleClick}
        disabled={isDisabled}
      >
        <UnderlineIcon />
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>
    <span>Underline</span>
    <span class="ml-1 text-xs">(cmd + u)</span>
  </TooltipContent>
</Tooltip>
