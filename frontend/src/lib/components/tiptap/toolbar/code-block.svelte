<script lang="ts">
  import CodeIcon from '@lucide/svelte/icons/code';
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

  const isActive = $derived(editor.isActive('codeBlock'));

  const isDisabled = $derived(
    !editor.can().chain().focus().toggleCodeBlock().run()
  );

  const handleClick = () => {
    editor.chain().focus().toggleCodeBlock().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Code Block"
    icon={CodeIcon}
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
          <CodeIcon />
        </Button.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Code Block</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
