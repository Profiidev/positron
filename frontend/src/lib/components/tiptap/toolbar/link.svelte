<script lang="ts">
  import LinkIcon from '@lucide/svelte/icons/link';
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import {
    Popover,
    PopoverContent,
    PopoverTrigger
  } from '@profidev/pleiades/components/ui/popover';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger
  } from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

  const isValidUrl = (url: string) => /^https?:\/\/\S+$/.test(url);

  const getUrlFromString = (str: string) => {
    if (isValidUrl(str)) {
      return str;
    }
    try {
      if (str.includes('.') && !str.includes(' ')) {
        return new URL(`https://${str}`).toString();
      }
    } catch {
      return null;
    }
  };

  let {
    editor,
    class: className,
    inOverflowMenu = false
  }: {
    editor: Editor;
    class?: string;
    inOverflowMenu?: boolean;
  } = $props();

  let link = $state('');

  const isActive = $derived(editor.isActive('link'));
  const isDisabled = $derived(
    !editor.can().chain().setLink({ href: '' }).run()
  );
  const linkHref = $derived(
    editor.getAttributes('link').href as string | undefined
  );

  $effect(() => {
    link = linkHref ?? '';
  });

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    const url = getUrlFromString(link);
    if (url) {
      editor.chain().focus().setLink({ href: url }).run();
    }
  };

  const handleRemove = () => {
    editor.chain().focus().unsetLink().run();
    link = '';
  };
</script>

{#snippet linkMenu()}
  <PopoverContent
    onCloseAutoFocus={(e) => e.preventDefault()}
    class="relative px-3 py-2.5"
  >
    <div class="relative">
      <form onsubmit={handleSubmit}>
        <p class="text-sm">Attach a link to the selected text</p>
        <div class="mt-3 flex flex-col items-end justify-end gap-3">
          <Input
            bind:value={link}
            class="w-full"
            placeholder="https://example.com"
          />
          <div class="flex items-center gap-3">
            {#if linkHref}
              <Button
                type="button"
                size="sm"
                variant="ghost"
                onclick={handleRemove}
              >
                <Trash2Icon class="mr-2" />
                Remove
              </Button>
            {/if}
            <Button size="sm" type="submit">
              {linkHref ? 'Update' : 'Confirm'}
            </Button>
          </div>
        </div>
      </form>
    </div>
  </PopoverContent>
{/snippet}

<Popover>
  {#if inOverflowMenu}
    <PopoverTrigger disabled={isDisabled}>
      {#snippet child({ props })}
        <ToolbarOverflowTrigger
          {...props}
          label="Link"
          icon={LinkIcon}
          hasSubmenu
          active={isActive}
          disabled={isDisabled}
          class={className}
        />
      {/snippet}
    </PopoverTrigger>
    {@render linkMenu()}
  {:else}
    <Tooltip>
      <TooltipTrigger>
        {#snippet child({ props })}
          <PopoverTrigger disabled={isDisabled}>
            {#snippet child({ props: triggerProps })}
              <Button
                {...props}
                {...triggerProps}
                variant="ghost"
                size="sm"
                type="button"
                class={cn(
                  'h-8 w-max cursor-pointer px-3 font-normal',
                  isActive && 'bg-accent',
                  className
                )}
              >
                <p class="mr-2 text-base">↗</p>
                <p class="decoration-gray-7 underline underline-offset-4">Link</p>
              </Button>
            {/snippet}
          </PopoverTrigger>
        {/snippet}
      </TooltipTrigger>
      <TooltipContent>
        <span>Link</span>
      </TooltipContent>
    </Tooltip>
    {@render linkMenu()}
  {/if}
</Popover>
