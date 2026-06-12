<script lang="ts">
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import HeadingIcon from '@lucide/svelte/icons/heading';
  import * as Button from '@profidev/pleiades/components/ui/button';
  import * as DropdownMenu from '@profidev/pleiades/components/ui/dropdown-menu';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

  const levels = [1, 2, 3, 4] as const;

  let {
    editor,
    class: className,
    inOverflowMenu = false
  }: {
    editor: Editor;
    class?: string;
    inOverflowMenu?: boolean;
  } = $props();

  const activeLevel = $derived(
    levels.find((level) => editor.isActive('heading', { level }))
  );
  const isHeadingActive = $derived(editor.isActive('heading'));
  const overflowLabel = $derived(activeLevel ? `Heading ${activeLevel}` : 'Normal');
</script>

{#snippet headingMenu()}
  <DropdownMenu.Content align="start" class="flex flex-col gap-1">
    <DropdownMenu.Item
      onclick={() => editor.chain().focus().setParagraph().run()}
      class={cn('flex h-fit items-center gap-2', !isHeadingActive && 'bg-accent')}
    >
      Normal
    </DropdownMenu.Item>
    {#each levels as level (level)}
      <DropdownMenu.Item
        onclick={() => editor.chain().focus().toggleHeading({ level }).run()}
        class={cn(
          'flex items-center gap-2',
          editor.isActive('heading', { level }) && 'bg-accent'
        )}
      >
        H{level}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
{/snippet}

{#if inOverflowMenu}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <ToolbarOverflowTrigger
          {...props}
          label={overflowLabel}
          icon={HeadingIcon}
          hasSubmenu
          class={className}
        />
      {/snippet}
    </DropdownMenu.Trigger>
    {@render headingMenu()}
  </DropdownMenu.Root>
{:else}
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props: triggerProps })}
              <Button.Root
                {...props}
                {...triggerProps}
                variant="ghost"
                size="sm"
                type="button"
                class={cn(
                  'h-8 w-max cursor-pointer gap-1 px-3 font-normal',
                  className
                )}
              >
                {activeLevel ? `H${activeLevel}` : 'Normal'}
                <ChevronDownIcon class="h-4 w-4" />
              </Button.Root>
            {/snippet}
          </DropdownMenu.Trigger>
          {@render headingMenu()}
        </DropdownMenu.Root>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>
      <span>Headings</span>
    </Tooltip.Content>
  </Tooltip.Root>
{/if}
