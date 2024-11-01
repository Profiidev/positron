<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { Nav } from "$lib/components/nav";
  import type { Option } from "$lib/components/nav";
  import { UserRound, Atom, PanelLeftClose, PanelLeftOpen } from "lucide-svelte";
  import { Separator } from "$lib/components/ui/separator";
  import { cn } from "$lib/utils";
  import { Button } from "$lib/components/ui/button";
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
</script>

<ModeWatcher />
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
