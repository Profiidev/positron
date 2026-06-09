<script lang="ts">
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import RepeatIcon from '@lucide/svelte/icons/repeat';
	import XIcon from '@lucide/svelte/icons/x';
	import { Button } from '@profidev/pleiades/components/ui/button';
	import { Checkbox } from '@profidev/pleiades/components/ui/checkbox';
	import { Input } from '@profidev/pleiades/components/ui/input';
	import { Label } from '@profidev/pleiades/components/ui/label';
	import {
		Popover,
		PopoverContent,
		PopoverTrigger
	} from '@profidev/pleiades/components/ui/popover';
	import { Separator } from '@profidev/pleiades/components/ui/separator';
	import { cn } from '@profidev/pleiades/utils';
	import type { Editor } from '@tiptap/core';

	let { editor }: { editor: Editor } = $props();

	let open = $state(false);
	let replacing = $state(false);
	let searchText = $state('');
	let replaceText = $state('');
	let checked = $state(false);

	const results = $derived(editor.storage.searchAndReplace.results);
	const selectedResult = $derived(editor.storage.searchAndReplace.selectedResult);

	function refreshSearchDecorations() {
		const { state, view } = editor;
		view.dispatch(state.tr);
	}

	function syncSearchToEditor() {
		const storage = editor.storage.searchAndReplace;
		storage.searchTerm = searchText;
		storage.replaceTerm = replaceText;
		storage.caseSensitive = checked;
		refreshSearchDecorations();
	}

	function resetSearchState() {
		searchText = '';
		replaceText = '';
		checked = false;
		replacing = false;

		const storage = editor.storage.searchAndReplace;
		storage.searchTerm = '';
		storage.replaceTerm = '';
		storage.caseSensitive = false;
		storage.selectedResult = 0;
		storage.results = [];
		refreshSearchDecorations();
	}

	function handleOpenChange(nextOpen: boolean) {
		open = nextOpen;
		if (!nextOpen) {
			resetSearchState();
		}
	}

	function handleSearchInput(value: string) {
		searchText = value;
		syncSearchToEditor();
	}

	function handleReplaceInput(value: string) {
		replaceText = value;
		syncSearchToEditor();
	}

	function handleCaseSensitiveChange(value: boolean) {
		checked = value;
		syncSearchToEditor();
	}

	const replace = () => editor.chain().replace().run();
	const replaceAll = () => editor.chain().replaceAll().run();
	const selectNext = () => editor.chain().selectNextResult().run();
	const selectPrevious = () => editor.chain().selectPreviousResult().run();
</script>

<Popover {open} onOpenChange={handleOpenChange}>
	<PopoverTrigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="ghost"
				size="sm"
				type="button"
				title="Search & Replace"
				class={cn('h-8 w-max px-3 font-normal')}
			>
				<RepeatIcon class="mr-2 h-4 w-4" />
				<p>Search & Replace</p>
			</Button>
		{/snippet}
	</PopoverTrigger>

	<PopoverContent
		align="end"
		onCloseAutoFocus={(e) => e.preventDefault()}
		class="relative flex w-[400px] px-3 py-2.5"
	>
		{#if !replacing}
			<div class={cn('relative flex items-center gap-1.5')}>
				<Input
					value={searchText}
					oninput={(e) => handleSearchInput(e.currentTarget.value)}
					class="w-48"
					placeholder="Search..."
				/>
				<span>
					{results.length === 0 ? selectedResult : selectedResult + 1}/{results.length}
				</span>
				<Button onclick={selectPrevious} size="icon" variant="ghost" class="size-7">
					<ArrowLeftIcon class="size-4" />
				</Button>
				<Button onclick={selectNext} size="icon" class="size-7" variant="ghost">
					<ArrowRightIcon class="h-4 w-4" />
				</Button>
				<Separator orientation="vertical" class="mx-0.5 h-7" />
				<Button
					onclick={() => {
						replacing = true;
					}}
					size="icon"
					class="size-7"
					variant="ghost"
				>
					<RepeatIcon class="h-4 w-4" />
				</Button>
				<Button
					onclick={() => handleOpenChange(false)}
					size="icon"
					class="size-7"
					variant="ghost"
				>
					<XIcon class="h-4 w-4" />
				</Button>
			</div>
		{:else}
			<div class={cn('relative w-full')}>
				<button
					type="button"
					onclick={() => handleOpenChange(false)}
					class="absolute right-3 top-3"
					aria-label="Close"
				>
					<XIcon class="h-4 w-4" />
				</button>
				<div class="flex w-full items-center gap-3">
					<Button
						size="icon"
						class="size-7 rounded-full"
						variant="ghost"
						onclick={() => {
							replacing = false;
						}}
					>
						<ArrowLeftIcon class="h-4 w-4" />
					</Button>
					<h2 class="text-sm font-medium">Search and replace</h2>
				</div>

				<div class="my-2 w-full">
					<div class="mb-3">
						<Label class="mb-1 text-xs text-muted-foreground">Search</Label>
						<Input
							value={searchText}
							oninput={(e) => handleSearchInput(e.currentTarget.value)}
							placeholder="Search..."
						/>
						{results.length === 0 ? selectedResult : selectedResult + 1}/{results.length}
					</div>
					<div class="mb-2">
						<Label class="mb-1 text-xs text-muted-foreground">Replace with</Label>
						<Input
							value={replaceText}
							oninput={(e) => handleReplaceInput(e.currentTarget.value)}
							class="w-full"
							placeholder="Replace..."
						/>
					</div>
					<div class="mt-3 flex items-center space-x-2">
						<Checkbox
							checked={checked}
							onCheckedChange={(value) => handleCaseSensitiveChange(value === true)}
							id="match_case"
						/>
						<Label
							for="match_case"
							class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
						>
							Match case
						</Label>
					</div>
				</div>

				<div class="actions mt-6 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Button onclick={selectPrevious} size="icon" class="h-7 w-7" variant="secondary">
							<ArrowLeftIcon class="h-4 w-4" />
						</Button>
						<Button onclick={selectNext} size="icon" class="h-7 w-7" variant="secondary">
							<ArrowRightIcon class="h-4 w-4" />
						</Button>
					</div>

					<div class="main-actions flex items-center gap-2">
						<Button
							size="sm"
							class="h-7 px-3 text-xs"
							variant="secondary"
							onclick={replaceAll}
						>
							Replace All
						</Button>
						<Button onclick={replace} size="sm" class="h-7 px-3 text-xs">Replace</Button>
					</div>
				</div>
			</div>
		{/if}
	</PopoverContent>
</Popover>
