<script lang="ts">
  import Redo2Icon from '@lucide/svelte/icons/redo-2';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

  let {
    editor,
    class: className,
    inOverflowMenu = false
  }: {
    editor: Editor;
    class?: string;
    inOverflowMenu?: boolean;
  } = $props();

  const isDisabled = $derived(!editor.can().chain().focus().redo().run());

  const handleClick = () => {
    editor.chain().focus().redo().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Redo"
    icon={Redo2Icon}
    disabled={isDisabled}
    onclick={handleClick}
    class={className}
  />
{:else}
  <Tooltip>
    <TooltipTrigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="ghost"
          size="icon"
          type="button"
          class={cn('cursor-pointer', className)}
          onclick={handleClick}
          disabled={isDisabled}
        >
          <Redo2Icon />
        </Button>
      {/snippet}
    </TooltipTrigger>
    <TooltipContent>
      <span>Redo</span>
    </TooltipContent>
  </Tooltip>
{/if}
