<script lang="ts">
  import {
    Sidebar,
    ModeWatcher,
    Toaster
  } from 'positron-components/components/ui';
  import '../app.css';
  import { page } from '$app/state';
  import SidebarApp from '$lib/components/nav/sidebar-app/sidebar-app.svelte';
  import { connect_updater } from '$lib/backend/ws/updater.svelte';
  import { test_token } from '$lib/backend/auth/other.svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';

  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ['/login', '/oauth', '/oauth/logout'];

  connect_updater();
  onMount(() => {
    test_token().then((valid) => {
      if (!valid && browser) {
        let url = page.url.pathname;
        if (url !== '/login') {
          goto('/login');
        }
      }
    });
  });
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if !noLayout.includes(page.url.pathname)}
  <Sidebar.Provider class="h-full">
    <SidebarApp />
    <Sidebar.Trigger class="absolute top-3 left-3 flex md:hidden" />
    <main class="flex h-full w-full">
      {@render children?.()}
    </main>
  </Sidebar.Provider>
{:else}
  {@render children?.()}
{/if}
