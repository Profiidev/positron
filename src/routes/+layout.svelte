<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { page } from "$app/stores";
  import { Handle, Pane, PaneGroup } from "$lib/components/ui/resizable";
  import { Nav } from "$lib/components/nav";
  import type { Option } from "$lib/components/nav";
  import { UserRound } from "lucide-svelte";

  const noLayout = ["/login"];
  const options: Option[] = [{
    title: "Test",
    icon: UserRound,
    selected: true,
  }, {
    title: "WW",
    icon: UserRound,
    selected: false,
  }];

  let isCollapsed = true;

  const onCollapse = () => {
    isCollapsed = true;
  };

  const onExpand = () => {
    isCollapsed = false;
  };
</script>

<ModeWatcher />
{#if !noLayout.includes($page.url.pathname)}
  <PaneGroup direction="horizontal" class="h-full w-full items-stretch">
    <Pane defaultSize={3} collapsedSize={3} collapsible minSize={10} maxSize={10} {onCollapse} {onExpand} class="flex flex-col">
      <Nav {isCollapsed} {options} />
      <Nav {isCollapsed} {options} class="mt-auto" />
    </Pane>
    <Handle withHandle />
    <Pane>
      <slot />
    </Pane>
  </PaneGroup>
{:else}
  <slot />
{/if}
