<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { LoaderCircle } from '@lucide/svelte';
  import { authorizeConfirm, logout, type UserInfo } from '$lib/client';
  import { toast } from '@profidev/pleiades/components/util/general';
  import SimpleAvatar from '$lib/components/SimpleAvatar.svelte';
  import { avatarUrl } from '$lib/permissions.svelte.js';

  let { data } = $props();

  let isLoading = $state(false);
  let user: UserInfo | undefined = $state();

  $effect(() => {
    data.user.then((u) => {
      user = u;
    });
  });

  const login = async (allow: boolean) => {
    if (!data.oauthOptions.code || !data.oauthOptions.name) {
      toast.error('There was an error while login in');
      return;
    }

    isLoading = true;
    let ret = await authorizeConfirm({
      query: {
        code: data.oauthOptions.code,
        allow
      }
    });
    isLoading = false;

    if (ret.response?.status === 401) {
      toast.error('You are not allowed to access this Application');
    } else if (ret.response?.status !== 200) {
      toast.error('There was an error while login in');
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
    goto(
      `/login?code=${data.oauthOptions.code}&name=${data.oauthOptions.name}`
    );
  };

  onMount(async () => {
    let settings = await data.settings;
    if (!settings || !settings.o_auth_instant_confirm) return;
    confirm();
  });
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title>Log in to {data.oauthOptions.name}</Card.Title>
      <Card.Description
        >Do you want to log in to {data.oauthOptions.name} with the account below?</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex w-100 items-center">
      {#if user}
        <SimpleAvatar
          src={user ? `${avatarUrl}/${user.uuid}` : ''}
          class="size-14"
        />
        <div class="ml-2 grid flex-1 text-left text-sm leading-tight">
          <span class="truncate text-lg font-semibold">{user.name}</span>
          <span class="truncate">{user.email}</span>
        </div>
        <Button variant="link" onclick={change} disabled={isLoading}
          >Change</Button
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
        <Button variant="secondary" onclick={cancel} disabled={isLoading}
          >Cancel</Button
        >
        <Button onclick={confirm} disabled={isLoading}>
          {#if isLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {/if}
          Confirm
        </Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
