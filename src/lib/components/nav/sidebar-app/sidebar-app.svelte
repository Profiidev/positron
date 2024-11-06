<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Atom, PanelLeftClose, PanelLeftOpen } from "lucide-svelte";
  import User from "./user.svelte";
  import Main from "./main.svelte";

  let sidebar = Sidebar.useSidebar();
  let isOpen = $derived(sidebar.props.open());
</script>

<Sidebar.Root collapsible="icon" variant="floating">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem class="flex-row flex">
        <Sidebar.MenuButton
          size="lg"
          class="max-w-0 data-[open=true]:max-w-44 overflow-hidden transition-all ease-linear"
          data-open={isOpen}
        >
          <div
            class="flex bg-sidebar-primary text-sidebar-primary-foreground aspect-square size-8 items-center justify-center rounded-lg"
          >
            <Atom />
          </div>
          <a class="font-semibold text-lg" href="/">Positron</a>
        </Sidebar.MenuButton>
        <Sidebar.MenuButton size="lg" class="size-12" onclick={sidebar.toggle}>
          {#snippet tooltipContent()}
            Positron
          {/snippet}
          <div
            data-open={!isOpen}
            class="ml-auto data-[open=true]:bg-sidebar-primary data-[open=true]:text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
          >
            {#if isOpen}
              <PanelLeftClose />
            {:else}
              <PanelLeftOpen />
            {/if}
          </div>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Header>
  <Sidebar.Separator />
  <Sidebar.Content>
    <Main />
  </Sidebar.Content>
  <Sidebar.Separator />
  <Sidebar.Footer>
    <User />
  </Sidebar.Footer>
</Sidebar.Root>
