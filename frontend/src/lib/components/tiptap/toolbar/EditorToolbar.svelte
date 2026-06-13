<script lang="ts">
  import type { Editor } from '@tiptap/core';
  import * as Tooltip from '@profidev/pleiades/components/ui/tooltip';
  import ToolbarOverflowContainer from './toolbar-overflow-container.svelte';

  let { editor }: { editor: Editor } = $props();
</script>

<div class="bg-background sticky top-0 z-20 block w-full border-b">
  <Tooltip.Provider>
    <ToolbarOverflowContainer {editor} />
  </Tooltip.Provider>
</div>
