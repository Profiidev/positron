<script lang="ts">
  import { Toaster, Sidebar } from "positron-components/components/ui";
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import SidebarApp from "$lib/components/nav/sidebar-app/sidebar-app.svelte";
  import { connect_updater } from "$lib/backend/ws/updater.svelte";
  import { test_token } from "$lib/backend/auth/other.svelte";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { getTokenCookie } from "$lib/backend/cookie.svelte";
  import { get } from "svelte/store";
  import { PUBLIC_IS_APP } from "$env/static/public";
  import { browser } from "$app/environment";

  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ["/login", "/oauth", "/oauth/logout"];

  connect_updater();
  test_token().then((valid) => {
    if (!valid && browser) {
      let url = get(page).url.pathname;
      if (url !== "/login") {
        goto("/login");
      }
    }
  });

  onMount(() => {
    if (PUBLIC_IS_APP !== "true" || !browser) return;

    let url = get(page).url.pathname;
    if (!getTokenCookie() && url !== "/login") {
      goto("/login");
    }
  });
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if !noLayout.includes($page.url.pathname)}
  <Sidebar.Provider class="h-full">
    <SidebarApp />
    <Sidebar.Trigger class="absolute left-3 top-3 flex md:hidden" />
    <main class="w-full h-full flex">
      {@render children?.()}
    </main>
  </Sidebar.Provider>
{:else}
  {@render children?.()}
{/if}
