<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as Dropdown from "$lib/components/ui/dropdown-menu";
  import { ChevronsUpDown, LogOut, Settings } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Avatar from "$lib/components/util/avatar.svelte";
  import { logout } from "$lib/backend/auth/other.svelte";
  import { getProfileInfo } from "$lib/backend/account/info.svelte";

  let infoData = $derived(getProfileInfo());
  let sidebar = Sidebar.useSidebar();

  const settings = () => {
    goto("/account");
  };

  const logoutFn = async () => {
    await logout();
    goto("/login");
  };
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <Dropdown.Root>
      <Dropdown.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            {...props}
          >
            {#if infoData}
              <Avatar src={infoData.image} class="size-8" />
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-semibold">{infoData.name}</span>
                <span class="truncate text-xs">{infoData.email}</span>
              </div>
            {:else}
              <Skeleton class="size-8 rounded-full" />
              <div
                class="grid flex-1 text-left text-sm leading-tight space-y-1"
              >
                <Skeleton class="h-4" />
                <Skeleton class="h-3" />
              </div>
            {/if}
            <ChevronsUpDown class="ml-auto size-4" />
          </Sidebar.MenuButton>
        {/snippet}
      </Dropdown.Trigger>
      <Dropdown.Content
        class="w-[--bits-dropdown-menu-anchor-width] min-w-56 rounded-lg"
        side={sidebar.isMobile ? "bottom" : "right"}
        align="end"
        sideOffset={4}
      >
        <Dropdown.Group>
          <Dropdown.Item onclick={settings}>
            <Settings />
            Settings
          </Dropdown.Item>
        </Dropdown.Group>
        <Dropdown.Separator />
        <Dropdown.Group>
          <Dropdown.Item onclick={logoutFn}>
            <LogOut />
            Log out
          </Dropdown.Item>
        </Dropdown.Group>
      </Dropdown.Content>
    </Dropdown.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
