<script lang="ts">
  import { ModeWatcher } from '@profidev/pleiades/components/util/general';
  import { Toaster } from '@profidev/pleiades/components/ui/sonner';
  import '../app.css';
  import { connectWebsocket } from '$lib/backend/updater.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import {
    items,
    noSidebarPaths,
    noAuthPaths
  } from '$lib/components/nav.svelte';
  import { setMode } from 'mode-watcher';
  import { logout, refreshToken, testToken, type UserInfo } from '$lib/client';
  import Sidebar from '@profidev/pleiades/components/nav/sidebar/sidebar.svelte';
  import Atom from '@lucide/svelte/icons/atom';
  import { avatarUrl } from '$lib/permissions.svelte';

  // @ts-ignore this is injected at build time via Vite's define option
  let version = __version__;

  let { children, data } = $props();

  let user: UserInfo | undefined = $state();
  let blockRedirect = false;

  $effect(() => {
    data.user.then((u) => {
      user = u;
    });
  });

  onMount(() => {
    setMode('dark');
    testToken().then(async ({ data: dataRaw }) => {
      let { valid, exp_short } = (dataRaw as
        | { valid: boolean; exp_short: boolean }
        | undefined) ?? {
        valid: false,
        exp_short: false
      };
      // can also be undefined if there was an error
      if (valid === false) {
        if (!blockRedirect && !noAuthPaths.includes(page.route.id ?? '')) {
          if (data.oauthOptions.code && data.oauthOptions.name) {
            goto(
              '/login?code=' +
                data.oauthOptions.code +
                '&name=' +
                data.oauthOptions.name
            );
          } else if (page.route.id?.startsWith('/auth/')) {
            const challenge = data.auth.challenge
              ? `&challenge=${data.auth.challenge}`
              : '';
            goto(
              `/login?auth=${page.route.id?.replace('/auth/', '')}${challenge}`
            );
          } else {
            goto('/login');
          }
        }
      } else {
        if (exp_short) {
          refreshToken();
        }
        let user = await data.user;
        connectWebsocket(user.uuid);
      }
    });

    (async () => {
      let { data: status, error } = await data.setupStatus;
      if (error) return;

      if (!status?.is_setup && page.route.id !== '/setup') {
        blockRedirect = true;
        await goto('/setup');
        blockRedirect = false;
      }
    })();
  });
</script>

<ModeWatcher />
<Toaster position="top-right" closeButton={true} richColors={true} />

{#if noSidebarPaths.includes(page.route.id ?? '')}
  {@render children()}
{:else}
  <Sidebar
    {user}
    app_name="Positron"
    app_icon={Atom}
    iconClass="text-[#000057]"
    avatar={user ? `${avatarUrl}/${user.uuid}` : undefined}
    {version}
    {items}
    logout={async () => {
      let res = await logout();
      return {
        error: res.error ? 'err' : undefined
      };
    }}
  >
    {@render children()}
  </Sidebar>
{/if}
