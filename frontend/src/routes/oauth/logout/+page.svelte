<script lang="ts">
  import { Button, Card, Skeleton } from 'positron-components/components/ui';
  import { SimpleAvatar } from 'positron-components/components/util';
  import { goto } from '$app/navigation';
  import type { PageServerData } from './$types';
  import { userData } from '$lib/backend/account/info.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_logout = $derived(data.oauth_logout);

  let error = $state('');
  let infoData = $derived(userData.value?.[1]);

  const back = async () => {
    if (!oauth_logout) {
      error = 'There was an error';
    } else {
      window.location.href = oauth_logout.url;
    }
  };

  const cancel = () => {
    goto('/');
  };
</script>

<div class="flex h-full items-center justify-center">
  <Card.Root>
    <Card.Header>
      <Card.Title>Logged out of {oauth_logout?.name}</Card.Title>
      <Card.Description
        >Do you want to got back to {oauth_logout?.name} or to Positron?</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex items-center">
      {#if infoData}
        <SimpleAvatar src={infoData.image} class="size-14" />
        <div class="ml-2 grid flex-1 text-left text-sm leading-tight">
          <span class="truncate text-lg font-semibold">{infoData.name}</span>
          <span class="truncate">{infoData.email}</span>
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
      <span class="text-destructive truncate text-sm">{error}</span>
      <div class="flex w-full justify-between">
        <Button variant="secondary" onclick={cancel}>Cancel</Button>
        <Button onclick={back}>Back</Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
