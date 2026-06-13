<script lang="ts">
  import BoldIcon from '@lucide/svelte/icons/bold';
  import * as Button from '@profidev/pleiades/components/ui/button';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
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

  const isActive = $derived(editor.isActive('bold'));

  const isDisabled = $derived(!editor.can().chain().focus().toggleBold().run());

  const handleClick = () => {
    editor.chain().focus().toggleBold().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Bold"
    icon={BoldIcon}
    active={isActive}
    disabled={isDisabled}
    onclick={handleClick}
    class={className}
  />
{:else}
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button.Root
          {...props}
          variant="ghost"
          size="icon"
          type="button"
          class={cn('cursor-pointer', isActive && 'bg-accent', className)}
          onclick={handleClick}
          disabled={isDisabled}
        >
          <BoldIcon />
        </Button.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Bold</span>
      <span class="ml-1 text-xs">(cmd + b)</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
