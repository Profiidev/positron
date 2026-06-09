<script lang="ts">
	import ListOrderedIcon from '@lucide/svelte/icons/list-ordered';
	import { Button } from '@profidev/pleiades/components/ui/button';
	import { Tooltip, TooltipContent, TooltipTrigger } from '@profidev/pleiades/components/ui/tooltip';
	import { cn } from '@profidev/pleiades/utils';
	import type { Editor } from '@tiptap/core';

	let { editor, class: className }: { editor: Editor; class?: string } = $props();

	const isActive = $derived(editor.isActive('orderedList'));
	const isDisabled = $derived(!editor.can().chain().focus().toggleOrderedList().run());

	function handleClick() {
		editor.chain().focus().toggleOrderedList().run();
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
				class={cn('h-8 w-8 p-0 sm:h-9 sm:w-9', isActive && 'bg-accent', className)}
				onclick={handleClick}
				disabled={isDisabled}
			>
				<ListOrderedIcon class="h-4 w-4" />
			</Button>
		{/snippet}
	</TooltipTrigger>
	<TooltipContent>
		<span>Ordered list</span>
	</TooltipContent>
</Tooltip>
