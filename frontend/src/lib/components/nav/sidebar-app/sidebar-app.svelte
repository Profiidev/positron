<script lang="ts">
  import { Sidebar, ScrollArea } from 'positron-components/components/ui';
  import { Atom, PanelLeftClose, PanelLeftOpen } from '@lucide/svelte';
  import User from './user.svelte';
  import Main from './main.svelte';

  let sidebar = Sidebar.useSidebar();
  let isOpen = $derived(sidebar.props.open());
</script>

<Sidebar.Root collapsible="icon" variant="floating">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem class="flex flex-row">
        <Sidebar.MenuButton
          size="lg"
          class="max-w-52 overflow-hidden transition-all ease-linear data-[open=true]:max-w-40 md:max-w-0"
          data-open={isOpen}
        >
          <div
            class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
          >
            <Atom />
          </div>
          <a class="text-lg font-semibold" href="/">Positron</a>
        </Sidebar.MenuButton>
        <Sidebar.MenuButton
          size="lg"
          class="ml-auto size-12"
          onclick={sidebar.toggle}
          aria-label="Open/Close Sidebar"
        >
          {#snippet tooltipContent()}
            Positron
          {/snippet}
          <div
            data-open={!isOpen}
            class="data-[open=true]:bg-sidebar-primary data-[open=true]:text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
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
    <ScrollArea.ScrollArea orientation={'vertical'}>
      <Main />
    </ScrollArea.ScrollArea>
  </Sidebar.Content>
  <Sidebar.Separator />
  <Sidebar.Footer>
    <User />
  </Sidebar.Footer>
</Sidebar.Root>
