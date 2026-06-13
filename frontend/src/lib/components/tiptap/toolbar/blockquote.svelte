<script lang="ts">
  import TextQuoteIcon from '@lucide/svelte/icons/text-quote';
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

  const isActive = $derived(editor.isActive('blockquote'));

  const isDisabled = $derived(
    !editor.can().chain().focus().toggleBlockquote().run()
  );

  const handleClick = () => {
    editor.chain().focus().toggleBlockquote().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Blockquote"
    icon={TextQuoteIcon}
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
          <TextQuoteIcon />
        </Button.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Blockquote</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
