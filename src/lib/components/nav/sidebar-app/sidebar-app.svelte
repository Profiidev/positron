<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Atom } from "lucide-svelte";
  import User from "./user.svelte";
  import Main from "./main.svelte";
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          size="lg"
          class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
        >
          {#snippet tooltipContent()}
            Positron
          {/snippet}
          <div
            class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
          >
            <Atom />
          </div>
          <a class="font-semibold text-lg" href="/">Positron</a>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Header>
  <Sidebar.Content>
    <Main />
  </Sidebar.Content>
  <Sidebar.Footer>
    <User />
  </Sidebar.Footer>
  <Sidebar.Rail />
</Sidebar.Root>
