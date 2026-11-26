<script lang="ts">
  import { ModeWatcher } from '@profidev/pleiades/components/util/general';
  import { Toaster } from '@profidev/pleiades/components/ui/sonner';
  import '../app.css';
  import {
    connectWebsocket,
    disconnectWebsocket
  } from '#lib/backend/updater.svelte.ts';
  import { onMount } from 'svelte';
  import { beforeNavigate, goto } from '$app/navigation';
  import { page, updated } from '$app/state';
  import * as Alert from '@profidev/pleiades/components/ui/alert';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import {
    items,
    noSidebarPaths,
    noAuthPaths
  } from '#lib/components/nav.svelte.ts';
  import {
    logout,
    refreshToken,
    testToken,
    type UserInfo
  } from '#lib/client/index.ts';
  import Sidebar from '@profidev/pleiades/components/nav/sidebar/sidebar.svelte';
  import Atom from '@lucide/svelte/icons/atom';
  import { avatarUrl } from '#lib/permissions.svelte.ts';
  import { buildLoginUrl } from '#lib/redirect.ts';
  import { getSessionMeta } from '@profidev/pleiades/util/info.svelte';

  // @ts-ignore this is injected at build time via Vite's define option
  let version = __version__;

  let { children, data } = $props();

  let user: UserInfo | undefined = $state();
  let blockRedirect = false;
  let updateDismissed = $state(false);

  // pending update: make next client-side navigation a full page load
  beforeNavigate(({ willUnload, to }) => {
    if (updated.current && !willUnload && to?.url) {
      location.href = to.url.href;
    }
  });

  $effect(() => {
    data.user.then((u) => {
      user = u;
    });
  });

  onMount(() => {
    testToken({ body: getSessionMeta() }).then(async ({ data: dataRaw }) => {
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
            goto(buildLoginUrl(page.url.pathname + page.url.search));
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

{#if updated.current && !updateDismissed}
  <Alert.Root class="fixed right-4 bottom-4 z-50 w-80 shadow-lg">
    <RefreshCw />
    <Alert.Title>Update available</Alert.Title>
    <Alert.Description>
      A new version has been deployed.
      <div class="mt-2 flex w-full justify-end gap-2">
        <Button
          variant="outline"
          size="sm"
          class="cursor-pointer"
          onclick={() => (updateDismissed = true)}>Later</Button
        >
        <Button
          size="sm"
          class="cursor-pointer"
          onclick={() => location.reload()}>Reload</Button
        >
      </div>
    </Alert.Description>
  </Alert.Root>
{/if}

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
      if (!res.error) {
        disconnectWebsocket();
      }

      return {
        error: res.error ? 'err' : undefined
      };
    }}
  >
    {@render children()}
  </Sidebar>
{/if}
