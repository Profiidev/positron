<script lang="ts" module>
  import { cn } from '@profidev/pleiades/utils';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  export type MobileToolbarItemProps = HTMLButtonAttributes & {
    active?: boolean;
    closeDrawer?: () => void;
  };
</script>

<script lang="ts">
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import {
    Drawer,
    DrawerContent,
    DrawerHeader,
    DrawerTitle,
    DrawerTrigger
  } from '@profidev/pleiades/components/ui/drawer';
  import type { Snippet } from 'svelte';

  let {
    label,
    class: className,
    children
  }: {
    label: string;
    class?: string;
    children: Snippet<[{ closeDrawer: () => void }]>;
  } = $props();

  let isOpen = $state(false);

  function closeDrawer() {
    isOpen = false;
  }
</script>

<Drawer bind:open={isOpen}>
  <DrawerTrigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        size="sm"
        type="button"
        class={cn('h-8 w-max gap-1 px-3 font-normal', className)}
      >
        {label}
        <ChevronDownIcon class="h-4 w-4" />
      </Button>
    {/snippet}
  </DrawerTrigger>
  <DrawerContent>
    <DrawerHeader>
      <DrawerTitle class="text-start">{label}</DrawerTitle>
    </DrawerHeader>
    <div class="flex flex-col p-4">
      {@render children({ closeDrawer })}
    </div>
  </DrawerContent>
</Drawer>
