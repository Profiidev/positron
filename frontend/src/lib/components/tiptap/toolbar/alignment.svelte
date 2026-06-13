<script lang="ts">
  import AlignCenterIcon from '@lucide/svelte/icons/align-center';
  import AlignJustifyIcon from '@lucide/svelte/icons/align-justify';
  import AlignLeftIcon from '@lucide/svelte/icons/align-left';
  import AlignRightIcon from '@lucide/svelte/icons/align-right';
  import CheckIcon from '@lucide/svelte/icons/check';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import * as Button from '@profidev/pleiades/components/ui/button';
  import * as DropdownMenu from '@profidev/pleiades/components/ui/dropdown-menu';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
  import type { Editor } from '@tiptap/core';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

  let {
    editor,
    inOverflowMenu = false
  }: {
    editor: Editor;
    inOverflowMenu?: boolean;
  } = $props();

  const alignmentOptions = [
    { name: 'Left Align', value: 'left', icon: AlignLeftIcon },
    { name: 'Center Align', value: 'center', icon: AlignCenterIcon },
    { name: 'Right Align', value: 'right', icon: AlignRightIcon },
    { name: 'Justify Align', value: 'justify', icon: AlignJustifyIcon }
  ] as const;

  const handleAlign = (value: string) => {
    editor.chain().focus().setTextAlign(value).run();
  };

  const currentTextAlign = $derived.by(() => {
    if (editor.isActive({ textAlign: 'left' })) return 'left';
    if (editor.isActive({ textAlign: 'center' })) return 'center';
    if (editor.isActive({ textAlign: 'right' })) return 'right';
    if (editor.isActive({ textAlign: 'justify' })) return 'justify';
    return 'left';
  });

  const currentOption = $derived(
    alignmentOptions.find((option) => option.value === currentTextAlign) ??
      alignmentOptions[0]
  );
</script>

{#snippet alignmentMenu()}
  <DropdownMenu.Content
    loop
    onCloseAutoFocus={(e) => e.preventDefault()}
    class="w-42"
  >
    <DropdownMenu.Group class="w-40">
      {#each alignmentOptions as option, index (index)}
        {@const OptionIcon = option.icon}
        <DropdownMenu.Item onSelect={() => handleAlign(option.value)}>
          <span class="mr-2">
            <OptionIcon class="h-4 w-4" />
          </span>
          {option.name}
          {#if option.value === currentTextAlign}
            <CheckIcon class="ml-auto h-4 w-4" />
          {/if}
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Group>
  </DropdownMenu.Content>
{/snippet}

{#if inOverflowMenu}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        {@const CurrentIcon = currentOption.icon}
        <ToolbarOverflowTrigger
          {...props}
          label={currentOption.name}
          icon={CurrentIcon}
          hasSubmenu
        />
      {/snippet}
    </DropdownMenu.Trigger>
    {@render alignmentMenu()}
  </DropdownMenu.Root>
{:else}
  <DropdownMenu.Root>
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          {@const CurrentIcon = currentOption.icon}
          <DropdownMenu.Trigger>
            {#snippet child({ props: triggerProps })}
              <Button.Root
                {...props}
                {...triggerProps}
                variant="ghost"
                size="sm"
                class="h-8 w-max cursor-pointer font-normal"
                type="button"
              >
                <span class="mr-2">
                  <CurrentIcon />
                </span>
                {currentOption.name}
                <ChevronDownIcon class="ml-2 h-4 w-4" />
              </Button.Root>
            {/snippet}
          </DropdownMenu.Trigger>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>Text Alignment</Tooltip.Content>
    </Tooltip.Root>
    {@render alignmentMenu()}
  </DropdownMenu.Root>
{/if}
