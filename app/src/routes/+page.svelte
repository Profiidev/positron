<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { logout } from '$lib/commands/auth.svelte';
  import { isConnected } from '$lib/updater/updater.svelte';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { goto } from '$app/navigation';
  import TipTab from '$lib/components/TipTab.svelte';

  const scanLoginCode = async () => {
    goto('/scan');
  };
</script>

<div class="grid h-full place-items-center">
  <div>
    <Button onclick={logout}>Logout</Button>
    <Button onclick={scanLoginCode}>Scan Login Code</Button>
    {#if !isConnected()}
      <Badge variant="destructive" class="ml-auto">Disconnected</Badge>
    {/if}
  </div>
  <TipTab id="8ae43409-4785-40e8-a668-1b570f520091" />
</div>