<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { goto } from '$app/navigation';
  import { LoaderCircle } from '@lucide/svelte';
  import { logout, requestAppCode, type UserInfo } from '$lib/client';
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

  const confirm = async () => {
    if (!data.auth.authType || data.auth.authType !== 'app' || !data.auth.challenge) {
      toast.error('There was an error while login in');
      return;
    }

    isLoading = true;

    let ret = await requestAppCode({
      body: {
        challenge: data.auth.challenge
      }
    });
    isLoading = false;

    if (ret.data && ret.response?.status === 200) {
      window.location.href = `positron://auth?code=${ret.data.code}`;
    } else {
      toast.error('There was an error while login in');
    }
  };

  const cancel = () => {
    goto('/');
  };

  const change = async () => {
    await logout();
    const challenge = data.auth.challenge
      ? `?challenge=${data.auth.challenge}`
      : '';
    goto(`/login?auth=${data.auth.authType}${challenge}`);
  };
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title>Log in to Positron App</Card.Title>
      <Card.Description
        >Do you want to log in with the account below?</Card.Description
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
