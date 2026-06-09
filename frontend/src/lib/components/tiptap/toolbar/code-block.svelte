<script lang="ts">
  import CodeIcon from '@lucide/svelte/icons/code';
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

  const isActive = $derived(editor.isActive('codeBlock'));
  const isDisabled = $derived(
    !editor.can().chain().focus().toggleCodeBlock().run()
  );

  function handleClick() {
    editor.chain().focus().toggleCodeBlock().run();
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
        class={cn(
          'h-8 w-8 p-0 sm:h-9 sm:w-9',
          isActive && 'bg-accent',
          className
        )}
        onclick={handleClick}
        disabled={isDisabled}
      >
        <CodeIcon class="h-4 w-4" />
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>
    <span>Code Block</span>
  </TooltipContent>
</Tooltip>
