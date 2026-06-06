<script lang="ts">
  import { ModeWatcher } from '@profidev/pleiades/components/util/general';
  import { Toaster } from '@profidev/pleiades/components/ui/sonner';
  import '../app.css';
  import { onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { startListener } from '$lib/updater.svelte';

  // @ts-ignore this is injected at build time via Vite's define option
  let version = __version__;

  let unlisten: UnlistenFn | undefined = undefined;

  onMount(() => {
    startListener().then((listener) => {
      unlisten = listener;
    });

    return () => {
      unlisten?.();
    };
  });

  let { children } = $props();
</script>

<ModeWatcher />
<Toaster
  position="top-right"
  closeButton={true}
  richColors={true}
  class="mt-10!"
/>

{@render children()}
