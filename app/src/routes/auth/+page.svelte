<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { isConnected } from '$lib/updater.svelte';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { startAuth } from '$lib/commands/auth.svelte.js';
  import { toast } from '@profidev/pleiades/components/util/general';

  type AuthStatus = 'Start' | 'Error';

  const { data } = $props();

  let status: AuthStatus = $state('Start');
  let isLoading = $state(false);

  const startAuth_ = async () => {
    isLoading = true;
    const challenge = await startAuth();
    if (!challenge) {
      status = 'Error';
      isLoading = false;
      toast.error('Failed to get auth challenge');
      return;
    }

    const url = new URL(data.url!);
    url.pathname = '/auth/app';
    url.searchParams.set('challenge', challenge);
    await openUrl(url);

    status = 'Start';
    isLoading = false;
  };
</script>

<div class="grid h-full place-items-center">
  <Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header>
      <Card.Title class="flex"
        >Login
        {#if !isConnected()}
          <Badge variant="destructive" class="ml-auto">Disconnected</Badge>
        {/if}
      </Card.Title>
    </Card.Header>
    <Card.Content class="flex items-center">
      <p>Start the authentication process</p>
      <Button
        class="ml-auto"
        onclick={startAuth_}
        disabled={isLoading}
        variant={status === 'Error' ? 'destructive' : 'default'}
      >
        {#if isLoading}
          <Spinner />
        {:else if status === 'Error'}
          <RotateCcw />
        {/if}
        Login</Button
      >
    </Card.Content>
  </Card.Root>
</div>
