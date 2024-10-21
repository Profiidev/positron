<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { Nav } from "$lib/components/nav";
  import type { Option } from "$lib/components/nav";
  import { UserRound, Menu } from "lucide-svelte";
  import { Separator } from "$lib/components/ui/separator";
  import { cn } from "$lib/utils";
  import { Button } from "$lib/components/ui/button";

  const noLayout = ["/login"];
  const optionsCollapse: Option[] = [{
    title: "Positron",
    icon: Menu,
    selected: false,
    click: () => isCollapsed = true,
  }];
  const options: Option[] = [{
    title: "Test",
    icon: UserRound,
    selected: false,
    click: () => {},
  }];

  let isCollapsed = true;
</script>

<ModeWatcher />
{#if !noLayout.includes($page.url.pathname)}
  <div class={cn("absolute h-full flex flex-col w-52 bg-background border-r transition-transform duration-500", {
    "-translate-x-52": isCollapsed
  })}>
    <Nav options={optionsCollapse} />
    <Separator />
    <Nav {options} />
    <Nav {options} class="mt-auto" />
  </div>
  <div>
    <Button variant="outline" size="icon" on:click={() => isCollapsed = false}>
      <Menu />
    </Button>
  </div>
  <slot />
{:else}
  <slot />
{/if}
