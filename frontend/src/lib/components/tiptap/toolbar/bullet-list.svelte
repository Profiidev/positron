<script lang="ts">
  import ListIcon from '@lucide/svelte/icons/list';
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

  const isActive = $derived(editor.isActive('bulletList'));

  const isDisabled = $derived(
    !editor.can().chain().focus().toggleBulletList().run()
  );

  const handleClick = () => {
    editor.chain().focus().toggleBulletList().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Bullet List"
    icon={ListIcon}
    active={isActive}
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
          class={cn('cursor-pointer', isActive && 'bg-accent', className)}
          onclick={handleClick}
          disabled={isDisabled}
        >
          <ListIcon />
        </Button>
      {/snippet}
    </TooltipTrigger>
    <TooltipContent>
      <span>Bullet List</span>
    </TooltipContent>
  </Tooltip>
{/if}
