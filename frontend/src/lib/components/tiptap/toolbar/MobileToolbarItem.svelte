<script lang="ts">
  import { cn } from '@profidev/pleiades/utils';
  import type { MobileToolbarItemProps } from './mobile-toolbar-group.svelte';

  let {
    class: className,
    active = false,
    closeDrawer,
    onclick,
    children,
    ...restProps
  }: MobileToolbarItemProps & {
    children?: import('svelte').Snippet;
  } = $props();

  function handleClick(
    e: MouseEvent & { currentTarget: EventTarget & HTMLButtonElement }
  ) {
    onclick?.(e);
    setTimeout(() => {
      closeDrawer?.();
    }, 100);
  }
</script>

<button
  class={cn(
    'hover:bg-accent flex w-full items-center rounded-md px-4 py-2 text-sm transition-colors',
    active && 'bg-accent',
    className
  )}
  onclick={handleClick}
  {...restProps}
>
  {@render children?.()}
</button>
