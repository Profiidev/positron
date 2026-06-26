<script lang="ts">
  import ListTodo from '@lucide/svelte/icons/list-todo';
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

  const isActive = $derived(editor.isActive('taskList'));

  const isDisabled = $derived(
    !editor.can().chain().focus().toggleTaskList().run()
  );

  const handleClick = () => {
    editor.chain().focus().toggleTaskList().run();
  };
</script>

{#if inOverflowMenu}
  <ToolbarOverflowTrigger
    label="Task List"
    icon={ListTodo}
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
          <ListTodo />
        </Button.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Task List</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
