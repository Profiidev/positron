<script lang="ts">
  import AlignCenterIcon from '@lucide/svelte/icons/align-center';
  import AlignJustifyIcon from '@lucide/svelte/icons/align-justify';
  import AlignLeftIcon from '@lucide/svelte/icons/align-left';
  import AlignRightIcon from '@lucide/svelte/icons/align-right';
  import CheckIcon from '@lucide/svelte/icons/check';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuTrigger
  } from '@profidev/pleiades/components/ui/dropdown-menu';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
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
  <DropdownMenuContent
    loop
    onCloseAutoFocus={(e) => e.preventDefault()}
    class="w-42"
  >
    <DropdownMenuGroup class="w-40">
      {#each alignmentOptions as option, index (index)}
        {@const OptionIcon = option.icon}
        <DropdownMenuItem onSelect={() => handleAlign(option.value)}>
          <span class="mr-2">
            <OptionIcon class="h-4 w-4" />
          </span>
          {option.name}
          {#if option.value === currentTextAlign}
            <CheckIcon class="ml-auto h-4 w-4" />
          {/if}
        </DropdownMenuItem>
      {/each}
    </DropdownMenuGroup>
  </DropdownMenuContent>
{/snippet}

{#if inOverflowMenu}
  <DropdownMenu>
    <DropdownMenuTrigger>
      {#snippet child({ props })}
        {@const CurrentIcon = currentOption.icon}
        <ToolbarOverflowTrigger
          {...props}
          label={currentOption.name}
          icon={CurrentIcon}
          hasSubmenu
        />
      {/snippet}
    </DropdownMenuTrigger>
    {@render alignmentMenu()}
  </DropdownMenu>
{:else}
  <DropdownMenu>
    <Tooltip>
      <TooltipTrigger>
        {#snippet child({ props })}
          {@const CurrentIcon = currentOption.icon}
          <DropdownMenuTrigger>
            {#snippet child({ props: triggerProps })}
              <Button
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
              </Button>
            {/snippet}
          </DropdownMenuTrigger>
        {/snippet}
      </TooltipTrigger>
      <TooltipContent>Text Alignment</TooltipContent>
    </Tooltip>
    {@render alignmentMenu()}
  </DropdownMenu>
{/if}
