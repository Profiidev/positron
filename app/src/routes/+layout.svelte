<script lang="ts">
  import { ModeWatcher } from '@profidev/pleiades/components/util/general';
  import { Toaster } from '@profidev/pleiades/components/ui/sonner';
  import '../app.css';
  import { onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { authStatusState, setupStatusState } from '$lib/updater/state.svelte';
  import { startListener } from '$lib/updater/updater.svelte';

  const setupStatus = $derived(setupStatusState.value);
  const authStatus = $derived(authStatusState.value);

  $effect(() => {
    if (
      (!setupStatus || !setupStatus.url) &&
      page.route.id !== '/setup' &&
      setupStatus !== null
    ) {
      goto('/setup');
    }
  });

  $effect(() => {
    if (
      authStatus !== undefined &&
      !authStatus &&
      page.route.id !== '/auth' &&
      page.route.id !== '/setup' &&
      authStatus !== null
    ) {
      goto('/auth');
    }
  });

  $effect(() => {
    if (
      authStatus &&
      setupStatus?.url &&
      (page.route.id === '/auth' || page.route.id === '/setup')
    ) {
      goto('/');
    }
  });

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
