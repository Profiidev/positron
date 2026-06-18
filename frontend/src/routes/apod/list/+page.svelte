<script lang="ts">
  import * as Tabs from '@profidev/pleiades/components/ui/tabs';
  import * as ScrollArea from '@profidev/pleiades/components/ui/scroll-area';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { DateTime as D } from '@profidev/pleiades/util/time.svelte';
  import type { ApodInfo } from '$lib/client';
  import { apodImageUrl } from '$lib/permissions.svelte.js';
  import { goto } from '$app/navigation';

  const { data } = $props();

  let apods: ApodInfo[] | undefined = $state();

  $effect(() => {
    data.apodList.then(async (list) => {
      apods = list;
    });
  });

  const itemClick = (date: string) => {
    let dateVal = new Date(date);
    goto(`/apod/${dateVal.toISOString().split('T')[0]}`);
  };
</script>

<Tabs.Root
  value="library"
  class="flex max-h-screen grow p-4"
  onValueChange={() => {
    goto(`/apod/${new Date().toISOString().split('T')[0]}`);
  }}
>
  <Tabs.List class="ml-10 w-fit md:ml-0">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="library" class="min-h-0 flex-1">
    <ScrollArea.ScrollArea orientation={'vertical'} class="h-full">
      <div
        class="grid w-full grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-3"
      >
        {#if apods}
          {#each apods as apod}
            <div class="flex aspect-square w-72 flex-col">
              <Button
                variant="ghost"
                class="h-full grow"
                onclick={() => itemClick(apod.date.toString())}
              >
                <img
                  class="h-auto w-auto rounded object-contain"
                  src={`${apodImageUrl}?preview=true&date=${new Date(apod.date).toISOString().split('T')[0]}`}
                  alt="Apod"
                />
              </Button>
              <p class="mt-1 ml-4">{apod.title}</p>
              <p class="text-muted-foreground ml-4">by {apod.user.name}</p>
              <p class="text-muted-foreground ml-4">
                on {D.DateTime?.fromISO(apod.date.toString())
                  .setLocale('de')
                  .toLocaleString(D.DateTime.DATE_MED)}
              </p>
            </div>
          {:else}
            <div class="col-span-full flex h-32 items-center justify-center">
              No APODs selected
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea.ScrollArea>
  </Tabs.Content>
</Tabs.Root>
