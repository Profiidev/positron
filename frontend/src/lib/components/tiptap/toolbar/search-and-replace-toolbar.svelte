<script lang="ts">
  import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
  import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
  import CaseSensitiveIcon from '@lucide/svelte/icons/case-sensitive';
  import RegexIcon from '@lucide/svelte/icons/regex';
  import RepeatIcon from '@lucide/svelte/icons/repeat';
  import * as Button from '@profidev/pleiades/components/ui/button';
  import * as Input from '@profidev/pleiades/components/ui/input';
  import * as Popover from '@profidev/pleiades/components/ui/popover';
  import * as Toggle from '@profidev/pleiades/components/ui/toggle';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
  import { cn } from '@profidev/pleiades/utils';
  import type { Editor } from '@tiptap/core';
  import { isValidSearchPattern } from '../extensions/regex-check';
  import ToolbarOverflowTrigger from './toolbar-overflow-trigger.svelte';

  let {
    editor,
    inOverflowMenu = false
  }: {
    editor: Editor;
    inOverflowMenu?: boolean;
  } = $props();

  let open = $state(false);
  let replacing = $state(false);
  let searchText = $state('');
  let replaceText = $state('');
  let caseSensitive = $state(false);
  let useRegex = $state(false);

  const results = $derived(editor.storage.searchAndReplace.results);
  const selectedResult = $derived(
    editor.storage.searchAndReplace.selectedResult
  );
  const isInvalidRegex = $derived(
    !isValidSearchPattern(searchText, useRegex, caseSensitive)
  );

  const refreshSearchDecorations = () => {
    const { state, view } = editor;
    view.dispatch(state.tr);
  };

  const syncSearchToEditor = () => {
    const storage = editor.storage.searchAndReplace;
    storage.searchTerm = searchText;
    storage.replaceTerm = replaceText;
    storage.caseSensitive = caseSensitive;
    storage.useRegex = useRegex;
    refreshSearchDecorations();
  };

  const resetSearchState = () => {
    searchText = '';
    replaceText = '';
    caseSensitive = false;
    useRegex = false;
    replacing = false;

    const storage = editor.storage.searchAndReplace;
    storage.searchTerm = '';
    storage.replaceTerm = '';
    storage.caseSensitive = false;
    storage.useRegex = false;
    storage.selectedResult = 0;
    storage.results = [];
    refreshSearchDecorations();
  };

  const handleOpenChange = (nextOpen: boolean) => {
    open = nextOpen;
    if (!nextOpen) {
      resetSearchState();
    }
  };

  const handleSearchInput = (value: string) => {
    searchText = value;
    syncSearchToEditor();
  };

  const handleReplaceInput = (value: string) => {
    replaceText = value;
    syncSearchToEditor();
  };

  const handleCaseSensitiveChange = (pressed: boolean) => {
    caseSensitive = pressed;
    syncSearchToEditor();
  };

  const handleUseRegexChange = (pressed: boolean) => {
    useRegex = pressed;
    syncSearchToEditor();
  };

  const replace = () => editor.chain().replace().run();
  const replaceAll = () => editor.chain().replaceAll().run();
  const selectNext = () => editor.chain().selectNextResult().run();
  const selectPrevious = () => editor.chain().selectPreviousResult().run();
</script>

{#snippet searchMenu()}
  <Popover.Content
    align="end"
    onCloseAutoFocus={(e) => e.preventDefault()}
    class="flex w-[412px] flex-col gap-1.5 px-3 py-2.5"
  >
    <div class="flex items-center gap-1.5">
      <Input.Root
        value={searchText}
        oninput={(e) => handleSearchInput(e.currentTarget.value)}
        class="w-48"
        placeholder="Search..."
        aria-invalid={isInvalidRegex}
        title={isInvalidRegex ? 'Invalid regular expression' : undefined}
      />
      <span class="text-muted-foreground shrink-0 text-xs tabular-nums">
        {results.length === 0
          ? selectedResult
          : selectedResult + 1}/{results.length}
      </span>
      <Button.Root
        onclick={selectPrevious}
        size="icon"
        variant="ghost"
        class="size-7"
        type="button"
      >
        <ArrowLeftIcon class="size-4" />
      </Button.Root>
      <Button.Root
        onclick={selectNext}
        size="icon"
        variant="ghost"
        class="size-7"
        type="button"
      >
        <ArrowRightIcon class="size-4" />
      </Button.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Toggle.Root
              {...props}
              pressed={caseSensitive}
              onPressedChange={handleCaseSensitiveChange}
              size="sm"
              variant="default"
              aria-label="Match case"
              class="size-7"
            >
              <CaseSensitiveIcon class="size-4" />
            </Toggle.Root>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>Match case</Tooltip.Content>
      </Tooltip.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Toggle.Root
              {...props}
              pressed={useRegex}
              onPressedChange={handleUseRegexChange}
              size="sm"
              variant="default"
              aria-label="Use regular expression"
              class="size-7"
            >
              <RegexIcon class="size-4" />
            </Toggle.Root>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>Use regular expression</Tooltip.Content>
      </Tooltip.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Toggle.Root
              {...props}
              bind:pressed={replacing}
              size="sm"
              variant="default"
              aria-label="Replace mode"
              class="size-7"
            >
              <RepeatIcon class="size-4" />
            </Toggle.Root>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>Replace</Tooltip.Content>
      </Tooltip.Root>
    </div>

    {#if replacing}
      <div class="flex items-center gap-1.5">
        <Input.Root
          value={replaceText}
          oninput={(e) => handleReplaceInput(e.currentTarget.value)}
          class="w-48"
          placeholder="Replace..."
        />
        <Button.Root
          onclick={replace}
          size="sm"
          class="ml-auto h-7 px-3 text-xs"
          type="button"
        >
          Replace
        </Button.Root>
        <Button.Root
          onclick={replaceAll}
          size="sm"
          variant="secondary"
          class="h-7 px-3 text-xs"
          type="button"
        >
          Replace All
        </Button.Root>
      </div>
    {/if}
  </Popover.Content>
{/snippet}

<Popover.Root {open} onOpenChange={handleOpenChange}>
  {#if inOverflowMenu}
    <Popover.Trigger>
      {#snippet child({ props })}
        <ToolbarOverflowTrigger
          {...props}
          label="Search & Replace"
          icon={RepeatIcon}
          hasSubmenu
        />
      {/snippet}
    </Popover.Trigger>
    {@render searchMenu()}
  {:else}
    <Popover.Trigger>
      {#snippet child({ props })}
        <Button.Root
          {...props}
          variant="ghost"
          size="sm"
          type="button"
          title="Search & Replace"
          class={cn('ml-auto h-8 w-max px-3 font-normal')}
        >
          <RepeatIcon class="mr-2 h-4 w-4" />
          <p>Search & Replace</p>
        </Button.Root>
      {/snippet}
    </Popover.Trigger>
    {@render searchMenu()}
  {/if}
</Popover.Root>
