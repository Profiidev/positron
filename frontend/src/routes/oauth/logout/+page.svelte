<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { goto } from '$app/navigation';
  import type { UserInfo } from '$lib/client';
  import SimpleAvatar from '$lib/components/SimpleAvatar.svelte';
  import { avatarUrl } from '$lib/permissions.svelte.js';
  import { toast } from '@profidev/pleiades/components/util/general';

  let { data } = $props();

  let user: UserInfo | undefined = $state();

  $effect(() => {
    data.user.then((u) => {
      user = u;
    });
  });

  const back = async () => {
    if (!data.oauthLogout) {
      toast.error('There was an error');
    } else {
      window.location.href = data.oauthLogout.url;
    }
  };

  const cancel = () => {
    goto('/');
  };
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title>Logged out of {data.oauthLogout?.name}</Card.Title>
      <Card.Description
        >Do you want to got back to {data.oauthLogout?.name} or to Positron?</Card.Description
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
        <Button variant="outline" onclick={cancel} class="cursor-pointer"
          >To Positron
        </Button>
        <Button onclick={back} class="cursor-pointer">Log back in</Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
