<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { UserRound, Atom, PanelLeftClose, PanelLeftOpen } from "lucide-svelte";
  import { Separator } from "$lib/components/ui/separator";
  import { cn } from "$lib/utils";
  import { Button } from "$lib/components/ui/button";
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { goto } from "$app/navigation";
  import type { Option } from "$lib/components/nav/nav.svelte";
  import Nav from "$lib/components/nav/nav.svelte";
    import { Toaster } from "$lib/components/ui/sonner";
  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ["/login"];
  const optionsCollapse: Option[] = [{
    title: "Positron",
    icon: Atom,
    selected: false,
    click: () => {},
  }];
  const options: Option[] = [{
    title: "Test",
    icon: UserRound,
    selected: false,
    click: () => {},
  }];

  let isCollapsed = $state(true);

  if(!get_token(TokenType.Auth)) {
    goto("/login", {
      replaceState: true,
    });
  }
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if !noLayout.includes($page.url.pathname)}
  {#if !isCollapsed}
    <div aria-hidden="true" class="w-full h-full absolute" onkeypress={() => {}} onclick={() => isCollapsed = true}></div>
  {/if}
  <div class={cn("absolute h-full flex flex-col w-52 bg-background border-r transition-transform duration-500", {
    "-translate-x-52": isCollapsed
  })}>
    <div class="relative">
      <Button variant="outline" size="icon" class={cn("absolute left-48 size-9 top-2 transition-transform duration-500", {
        "translate-x-4": isCollapsed,
      })} onclick={() => isCollapsed = !isCollapsed}>
        {#if isCollapsed}
          <PanelLeftOpen class="size-5" />
        {:else}
          <PanelLeftClose class="size-5" />
        {/if}
      </Button>
    </div>
    <Nav options={optionsCollapse} />
    <Separator />
    <Nav {options} />
    <Nav {options} class="mt-auto" />
  </div>
  {@render children?.()}
{:else}
  {@render children?.()}
{/if}
