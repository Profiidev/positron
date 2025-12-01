<script lang="ts">
  import * as Sidebar from 'positron-components/components/ui/sidebar';
  // @ts-ignore
  import * as DropdownMenu from 'positron-components/components/ui/dropdown-menu';
  import { Skeleton } from 'positron-components/components/ui/skeleton';
  import SimpleAvatar from 'positron-components/components/util/simple-avatar.svelte';
  import { ChevronsUpDown, LogOut, Settings } from '@lucide/svelte';
  import { goto } from '$app/navigation';
  import { logout } from '$lib/backend/auth/other.svelte';
  import { userData } from '$lib/backend/account/info.svelte';

  let infoData = $derived(userData.value?.[1]);
  let sidebar = Sidebar.useSidebar();

  const settings = () => {
    goto('/account');
  };

  const logoutFn = async () => {
    await logout();
    goto('/login');
  };
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props }: { props: Record })}
          <Sidebar.MenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            {...props}
          >
            {#if infoData}
              <SimpleAvatar src={infoData.image} class="size-8" />
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-semibold">{infoData.name}</span>
                <span class="truncate text-xs">{infoData.email}</span>
              </div>
            {:else}
              <Skeleton class="size-8 rounded-full" />
              <div
                class="grid flex-1 space-y-1 text-left text-sm leading-tight"
              >
                <Skeleton class="h-4" />
                <Skeleton class="h-3" />
              </div>
            {/if}
            <ChevronsUpDown class="ml-auto size-4" />
          </Sidebar.MenuButton>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
        side={sidebar.isMobile ? 'bottom' : 'right'}
        align="end"
        sideOffset={4}
      >
        <DropdownMenu.Group>
          <DropdownMenu.Item onclick={settings}>
            <Settings />
            Settings
          </DropdownMenu.Item>
        </DropdownMenu.Group>
        <DropdownMenu.Separator />
        <DropdownMenu.Group>
          <DropdownMenu.Item onclick={logoutFn}>
            <LogOut />
            Log out
          </DropdownMenu.Item>
        </DropdownMenu.Group>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
