<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { goto } from '$app/navigation';
  import { logout } from '$lib/commands/auth.svelte';
  import { isConnected } from '$lib/updater/updater.svelte';
  import NotebookPen from '@lucide/svelte/icons/notebook-pen';
  import ScanLine from '@lucide/svelte/icons/scan-line';
  import LogOut from '@lucide/svelte/icons/log-out';
</script>

<nav class="bg-background flex items-center gap-1 border-b p-2">
  <Button variant="ghost" class="cursor-pointer" onclick={() => goto('/')}>
    <NotebookPen />
    Notes
  </Button>
  <Button variant="ghost" class="cursor-pointer" onclick={() => goto('/scan')}>
    <ScanLine />
    Scan Login
  </Button>
  {#if !isConnected()}
    <Badge variant="destructive">Disconnected</Badge>
  {/if}
  <Button
    variant="ghost"
    class="text-destructive ml-auto cursor-pointer"
    onclick={logout}
  >
    <LogOut />
    Logout
  </Button>
</nav>
