<script lang="ts">
  import Nav from '$lib/components/Nav.svelte';

  const { children } = $props();
</script>

<div class="flex h-full flex-col">
  <Nav />
  {@render children()}
</div>
