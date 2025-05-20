<script lang="ts">
  import { Button, Card, Skeleton } from 'positron-components/components/ui';
  import { SimpleAvatar } from 'positron-components/components/util';
  import { RequestError } from 'positron-components/backend';
  import { goto } from '$app/navigation';
  import { logout, oauth_auth } from '$lib/backend/auth/other.svelte';
  import type { PageServerData } from './$types';
  import { userData } from '$lib/backend/account/info.svelte';
  import { onMount } from 'svelte';
  import {
    user_settings,
    user_settings_get
  } from '$lib/backend/account/settings.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_params = $derived(data.oauth_params);

  let isLoading = $state(false);
  let error = $state('');
  let infoData = $derived(userData.value?.[1]);

  const login = async (allow: boolean) => {
    if (!oauth_params) {
      error = 'There was an error while login in';
      return;
    }

    error = '';
    isLoading = true;

    let ret = await oauth_auth(oauth_params, allow);

    isLoading = false;
    if (ret === RequestError.Other) {
      error = 'There was an error while login in';
    } else if (ret === RequestError.Unauthorized) {
      error = 'You are not allowed to access this Application';
    }
  };

  const confirm = () => {
    login(true);
  };

  const cancel = () => {
    login(false);
    goto('/');
  };

  const change = async () => {
    await logout();
    goto(`/login?code=${oauth_params?.code}&name=${oauth_params?.name}`);
  };

  onMount(async () => {
    let settings = await user_settings_get();
    if (!settings || !settings.o_auth_instant_confirm) return;
    confirm();
  });
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title>Log in to {oauth_params?.name}</Card.Title>
      <Card.Description
        >Do you want to log in to {oauth_params?.name} with the account below?</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex items-center w-100">
      {#if infoData}
        <SimpleAvatar src={infoData.image} class="size-14" />
        <div class="ml-2 grid flex-1 text-left text-sm leading-tight">
          <span class="truncate text-lg font-semibold">{infoData.name}</span>
          <span class="truncate">{infoData.email}</span>
        </div>
        <Button variant="link" onclick={change}>Change</Button>
      {:else}
        <Skeleton class="size-14 rounded-full" />
        <div class="ml-2 grid flex-1 space-y-2 text-left text-sm leading-tight">
          <Skeleton class="h-5 w-32 rounded-full" />
          <Skeleton class="h-3 w-32" />
        </div>
      {/if}
    </Card.Content>
    <Card.Footer class="flex flex-col">
      <span class="text-destructive mb-4 truncate text-sm">{error}</span>
      <div class="flex w-full justify-between">
        <Button variant="secondary" onclick={cancel}>Cancel</Button>
        <Button onclick={confirm}>Confirm</Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
