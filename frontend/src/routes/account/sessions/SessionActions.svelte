<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import Lock from '@lucide/svelte/icons/lock';
  import LogOut from '@lucide/svelte/icons/log-out';
  import type { SessionInfo } from '$lib/client';

  interface Props {
    session: SessionInfo;
    revoke: (session: SessionInfo) => void;
  }

  let { session, revoke }: Props = $props();
</script>

<div class="flex w-full justify-end mr-2">
  {#if session.current}
    <Button
      variant="outline"
      size="icon"
      class="cursor-default opacity-50"
      disabled
      aria-label="Current session"
    >
      <Lock class="size-4" />
    </Button>
  {:else}
    <Button
      variant="outline"
      size="icon"
      class="border-destructive text-destructive hover:text-destructive cursor-pointer"
      aria-label="Revoke session"
      onclick={() => revoke(session)}
    >
      <LogOut class="size-4" />
    </Button>
  {/if}
</div>
