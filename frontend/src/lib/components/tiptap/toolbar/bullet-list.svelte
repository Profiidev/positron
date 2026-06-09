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

  let { editor, class: className }: { editor: Editor; class?: string } =
    $props();

  const isActive = $derived(editor.isActive('bulletList'));
  const isDisabled = $derived(
    !editor.can().chain().focus().toggleBulletList().run()
  );

  function handleClick() {
    editor.chain().focus().toggleBulletList().run();
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
        <ListIcon />
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>
    <span>Bullet list</span>
  </TooltipContent>
</Tooltip>
