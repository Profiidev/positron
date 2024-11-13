<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { Toaster } from "$lib/components/ui/sonner";
  import SidebarApp from "$lib/components/nav/sidebar-app/sidebar-app.svelte";
  import * as Sidebar from "$lib/components/ui/sidebar";

  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ["/login", "/oauth"];
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if !noLayout.includes($page.url.pathname)}
  <Sidebar.Provider>
    <SidebarApp />
    <Sidebar.Trigger class="absolute left-3 top-3 flex md:hidden" />
    <main class="w-full h-full">
      {@render children?.()}
    </main>
  </Sidebar.Provider>
{:else}
  {@render children?.()}
{/if}
