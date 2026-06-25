<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { goto } from '$app/navigation';
  import SimpleAvatar from '$lib/components/SimpleAvatar.svelte';
  import LoaderCircle from '@lucide/svelte/icons/loader-circle';
  import { confirmCode, logout } from '$lib/commands/auth.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';
  import {
    setupStatusState,
    userAvatarState,
    userInfoState
  } from '$lib/updater/state.svelte.js';
  import { isConnected } from '$lib/updater/updater.svelte.js';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { openUrl } from '@tauri-apps/plugin-opener';

  const { data } = $props();

  const user = $derived(userInfoState.value);
  const avatar = $derived(userAvatarState.value);
  const setupStatus = $derived(setupStatusState.value);
  let isLoading = $state(false);

  const isAllowedRedirect = (redirect: string, serverUrl: string) => {
    try {
      return new URL(redirect).origin === new URL(serverUrl).origin;
    } catch {
      return false;
    }
  };

  const confirm = async () => {
    if (!data.code) return;
    isLoading = true;
    const result = await confirmCode(data.code);
    if (result) {
      toast.success('Login confirmed successfully.');
      if (
        data.redirect &&
        setupStatus?.url &&
        isAllowedRedirect(data.redirect, setupStatus.url)
      ) {
        await openUrl(data.redirect);
      }
      goto('/');
    } else {
      toast.error('Failed to confirm login.');
    }
    isLoading = false;
  };

  const cancel = () => {
    goto('/');
  };

  const change = async () => {
    await logout();
    goto(`/auth`);
  };
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex"
        >Log in to Positron App
        {#if !isConnected()}
          <Badge variant="destructive" class="ml-auto">Disconnected</Badge>
        {/if}
      </Card.Title>
      <Card.Description
        >Do you want to log in with the account below?</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex w-100 items-center">
      {#if user}
        <SimpleAvatar src={avatar ?? ''} class="size-14" />
        <div class="ml-2 grid flex-1 text-left text-sm leading-tight">
          <span class="truncate text-lg font-semibold">{user.name}</span>
          <span class="truncate">{user.email}</span>
        </div>
        <Button
          variant="link"
          onclick={change}
          disabled={isLoading}
          class="cursor-pointer">Change</Button
        >
      {:else}
        <Skeleton class="size-14 rounded-full" />
        <div class="ml-2 grid flex-1 space-y-2 text-left text-sm leading-tight">
          <Skeleton class="h-5 w-32 rounded-full" />
          <Skeleton class="h-3 w-32" />
        </div>
      {/if}
    </Card.Content>
    <Card.Footer class="flex flex-col">
      <div class="flex w-full justify-between">
        <Button
          variant="outline"
          onclick={cancel}
          disabled={isLoading}
          class="cursor-pointer">Cancel</Button
        >
        <Button onclick={confirm} disabled={isLoading} class="cursor-pointer">
          {#if isLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {/if}
          Confirm
        </Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
