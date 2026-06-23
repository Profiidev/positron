<script lang="ts">
  import { cn } from '@profidev/pleiades/utils';
  import { expiryClass, formatExpiry } from './session-utils';

  interface Props {
    expiresAt: Date;
  }

  let { expiresAt }: Props = $props();

  const formatted = $derived(formatExpiry(expiresAt));
</script>

<span
  class={cn(formatted.urgency !== 'normal' && expiryClass(formatted.urgency))}
>
  {formatted.text}
</span>
