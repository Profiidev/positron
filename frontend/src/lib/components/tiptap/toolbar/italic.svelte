<script lang="ts">
  import ItalicIcon from '@lucide/svelte/icons/italic';
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

  const isActive = $derived(editor.isActive('italic'));

  const isDisabled = $derived(
    !editor.can().chain().focus().toggleItalic().run()
  );

  const handleClick = () => {
    editor.chain().focus().toggleItalic().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Italic"
    icon={ItalicIcon}
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
          <ItalicIcon />
        </Button.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Italic</span>
      <span class="ml-1 text-xs">(cmd + i)</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
