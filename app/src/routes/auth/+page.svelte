<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { onMount } from 'svelte';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { isConnected } from '$lib/updater.svelte';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import { openUrl } from '@tauri-apps/plugin-opener';

  type AuthStatus = 'Start' | 'Error' | 'Success' | 'Finished' | 'SendError';

  const { data } = $props();

  let status: AuthStatus = $state('Start');
  let isLoading = $state(false);

  const startAuth = async () => {
    isLoading = true;
    const url = new URL(data.url!);
    url.pathname = '/auth/app';
    await openUrl(url);
    isLoading = false;
  };

  let error = $state(false);
  let code = $state('');

  const authCli = async () => {};

  onMount(authCli);
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
      {#if status === 'Start'}
        <p>Start the authentication process</p>
        <Button class="ml-auto" onclick={startAuth} disabled={isLoading}>
          {#if isLoading}
            <Spinner />
          {/if}
          Login</Button
        >
      {:else if status === 'Error'}
        <p class="text-red-500">Failed to get CLI auth code</p>
      {:else if status === 'Success'}
        <p>Trying to authenticate CLI with code:</p>
        <Input value={code} readonly class="my-2 w-full" />
        <p>If it is not working try to paste the code into the CLI manually.</p>
      {:else if status === 'SendError'}
        <p>Failed to send code. Please enter it manually:</p>
        <Input value={code} readonly class="my-2 w-full" />
      {:else if status === 'Finished'}
        <p>CLI authenticated successfully! You can now close this window.</p>
      {/if}
      {#if error}
        <Button variant="outline" class="mt-4 w-full" onclick={authCli}>
          <RotateCcw class="mr-2" />
          Try Again
        </Button>
      {/if}
    </Card.Content>
  </Card.Root>
</div>
