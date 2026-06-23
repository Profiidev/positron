<script lang="ts">
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import Monitor from '@lucide/svelte/icons/monitor';
  import Smartphone from '@lucide/svelte/icons/smartphone';

  interface Props {
    isApp: boolean;
  }

  let { isApp }: Props = $props();
</script>

<Badge variant="secondary" class="gap-1">
  {#if isApp}
    <Smartphone class="size-3.5" />
    App
  {:else}
    <Monitor class="size-3.5" />
    Browser
  {/if}
</Badge>
