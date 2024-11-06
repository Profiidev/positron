<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { goto } from "$app/navigation";
  import { Toaster } from "$lib/components/ui/sonner";
  import Nav from "$lib/components/nav/nav.svelte";
  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ["/login"];

  if (!get_token(TokenType.Auth)) {
    goto("/login", {
      replaceState: true,
    });
  }
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if !noLayout.includes($page.url.pathname)}
  <Nav />
  {@render children?.()}
{:else}
  {@render children?.()}
{/if}
