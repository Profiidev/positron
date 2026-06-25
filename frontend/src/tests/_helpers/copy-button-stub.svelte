<script lang="ts">
  import type { Snippet } from 'svelte';

  // Test stub for pleiades' CopyButton: bits-ui's click handling doesn't fire
  // under jsdom fireEvent, so we render the label children and surface the
  // `text` prop via data-copy-text for assertions.
  let { text, children }: { text: string; children?: Snippet } = $props();
</script>

<button name="copy" data-copy-text={text}>
  {@render children?.()}
</button>
