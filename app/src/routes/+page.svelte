<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Label } from '@profidev/pleiades/components/ui/label';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { logout } from '$lib/commands/auth.svelte';
  import { isConnected } from '$lib/updater/updater.svelte';
  import { Badge } from '@profidev/pleiades/components/ui/badge';

  let name = $state('');
  let greetMsg = $state('');

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke('greet', { name });
  }
</script>

<div class="grid h-full place-items-center">
  <div>
    <Label>{greetMsg}</Label>
    <Button onclick={greet}>Greet</Button>
    <Button onclick={logout}>Logout</Button>
    {#if !isConnected()}
      <Badge variant="destructive" class="ml-auto">Disconnected</Badge>
    {/if}
    <a href="/login?code=123">Test</a>
  </div>
</div>
