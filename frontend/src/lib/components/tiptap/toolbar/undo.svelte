<script lang="ts">
	import Undo2Icon from '@lucide/svelte/icons/undo-2';
	import { Button } from '@profidev/pleiades/components/ui/button';
	import { Tooltip, TooltipContent, TooltipTrigger } from '@profidev/pleiades/components/ui/tooltip';
	import { cn } from '@profidev/pleiades/utils';
	import type { Editor } from '@tiptap/core';

	let { editor, class: className }: { editor: Editor; class?: string } = $props();

	const isDisabled = $derived(!editor.can().chain().focus().undo().run());

	function handleClick() {
		editor.chain().focus().undo().run();
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
				class={cn('h-8 w-8 p-0 sm:h-9 sm:w-9', className)}
				onclick={handleClick}
				disabled={isDisabled}
			>
				<Undo2Icon class="h-4 w-4" />
			</Button>
		{/snippet}
	</TooltipTrigger>
	<TooltipContent>
		<span>Undo</span>
	</TooltipContent>
</Tooltip>
