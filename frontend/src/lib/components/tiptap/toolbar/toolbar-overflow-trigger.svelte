<script lang="ts">
  import type { Component, ComponentProps } from 'svelte';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { cn } from '@profidev/pleiades/utils';

  let {
    label,
    icon: Icon,
    hasSubmenu = false,
    active = false,
    class: className,
    ...buttonProps
  }: ComponentProps<typeof Button> & {
    label: string;
    icon: Component;
    hasSubmenu?: boolean;
    active?: boolean;
  } = $props();
</script>

<Button
  variant="ghost"
  size="sm"
  type="button"
  {...buttonProps}
  class={cn(
    'h-8 w-full justify-start gap-2 px-2 font-normal',
    active && 'bg-accent',
    className
  )}
>
  <Icon class="h-4 w-4 shrink-0" />
  <span class="min-w-0 flex-1 truncate text-left text-sm">{label}</span>
  {#if hasSubmenu}
    <ChevronRightIcon class="h-4 w-4 shrink-0 opacity-60" />
  {/if}
</Button>
