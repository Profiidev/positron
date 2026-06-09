<script lang="ts">
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import XIcon from '@lucide/svelte/icons/x';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import { Label } from '@profidev/pleiades/components/ui/label';
  import {
    Popover,
    PopoverClose,
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

  function isValidUrl(url: string) {
    return /^https?:\/\/\S+$/.test(url);
  }

  function getUrlFromString(str: string) {
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
  }

  let { editor, class: className }: { editor: Editor; class?: string } =
    $props();

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

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const url = getUrlFromString(link);
    if (url) {
      editor.chain().focus().setLink({ href: url }).run();
    }
  }

  function handleRemove() {
    editor.chain().focus().unsetLink().run();
    link = '';
  }
</script>

<Popover>
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
                'h-8 w-max px-3 font-normal',
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

  <PopoverContent
    onCloseAutoFocus={(e) => e.preventDefault()}
    class="relative px-3 py-2.5"
  >
    <div class="relative">
      <PopoverClose class="absolute top-3 right-3">
        <XIcon class="h-4 w-4" />
      </PopoverClose>
      <form onsubmit={handleSubmit}>
        <Label>Link</Label>
        <p class="text-gray-11 text-sm">Attach a link to the selected text</p>
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
                class="text-gray-11 h-8"
                variant="ghost"
                onclick={handleRemove}
              >
                <Trash2Icon class="mr-2 h-4 w-4" />
                Remove
              </Button>
            {/if}
            <Button size="sm" class="h-8" type="submit">
              {linkHref ? 'Update' : 'Confirm'}
            </Button>
          </div>
        </div>
      </form>
    </div>
  </PopoverContent>
</Popover>
