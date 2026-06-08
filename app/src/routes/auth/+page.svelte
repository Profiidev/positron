<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { startAuth } from '$lib/commands/auth.svelte.js';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { resetSetup } from '$lib/commands/setup.svelte.js';
  import { goto } from '$app/navigation';
  import { setupStatusState } from '$lib/updater/state.svelte';
  import { isConnected } from '$lib/updater/updater.svelte';

  type AuthStatus = 'Start' | 'Error';

  const setupStatus = $derived(setupStatusState.value);

  $effect(() => {
    if (!setupStatus?.url && setupStatus !== null) {
      goto('/setup');
    }
  });

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

    const url = new URL(setupStatus?.url!);
    url.pathname = '/auth/app';
    url.searchParams.set('challenge', challenge);
    await openUrl(url);

    status = 'Start';
    isLoading = false;
  };

  const resetSetup_ = async () => {
    const success = await resetSetup();
    if (!success) {
      toast.error('Failed to reset setup');
    } else {
      goto('/setup');
    }
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
    <Card.Content class="flex flex-col">
      <div class="mb-2 flex items-center">
        <p>
          URL: {setupStatus?.url}
        </p>
        <Button
          class="ml-auto cursor-pointer"
          variant="link"
          onclick={resetSetup_}
          disabled={isLoading}
        >
          Change</Button
        >
      </div>
      <div class="flex items-center">
        <p>Start the authentication process</p>
        <Button
          class="mr-2 ml-auto cursor-pointer"
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
      </div>
    </Card.Content>
  </Card.Root>
</div>
