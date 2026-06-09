<script lang="ts">
  import WrapTextIcon from '@lucide/svelte/icons/wrap-text';
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

  function handleClick() {
    editor.chain().focus().setHardBreak().run();
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
        class={cn('h-8 w-8 p-0 sm:h-9 sm:w-9', className)}
        onclick={handleClick}
      >
        <WrapTextIcon class="h-4 w-4" />
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>
    <span>Hard break</span>
  </TooltipContent>
</Tooltip>
