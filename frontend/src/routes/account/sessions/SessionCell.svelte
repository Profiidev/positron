<script lang="ts">
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import Monitor from '@lucide/svelte/icons/monitor';
  import Smartphone from '@lucide/svelte/icons/smartphone';
  import type { SessionInfo } from '$lib/client';
  import { sessionDisplayName, sessionSubtitle } from './session-utils';

  interface Props {
    session: SessionInfo;
  }

  let { session }: Props = $props();

  const Icon = $derived(session.is_app ? Smartphone : Monitor);
  const displayName = $derived(
    sessionDisplayName(session.name, session.is_app)
  );
  const subtitle = $derived(
    sessionSubtitle(session.application, session.operating_system)
  );
</script>

<div class="ml-1 flex min-w-48 items-center gap-3 py-1">
  <Icon class="text-muted-foreground mx-1 size-5 shrink-0" />
  <div class="min-w-0 space-y-0.5">
    <div class="flex items-center gap-2">
      <span class="font-medium">{displayName}</span>
      {#if session.current}
        <Badge
          class="bg-emerald-500/15 text-emerald-500 hover:bg-emerald-500/15"
        >
          This device
        </Badge>
      {/if}
    </div>
    <p class="text-muted-foreground truncate text-sm">{subtitle}</p>
  </div>
</div>
