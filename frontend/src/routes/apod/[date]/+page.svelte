<script lang="ts">
  import * as Tabs from '@profidev/pleiades/components/ui/tabs';
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import Datepicker from '@profidev/pleiades/components/util/datepicker.svelte';
  import {
    getLocalTimeZone,
    now,
    parseDate,
    type DateValue
  } from '@profidev/pleiades/util/time.svelte';
  import { LoaderCircle } from '@lucide/svelte';
  import type { GetInfoRes } from '$lib/client/types.gen.js';
  import { apodImageUrl } from '$lib/permissions.svelte.js';
  import { setGoodApod } from '$lib/client';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import { cn } from '@profidev/pleiades/utils';

  const { data } = $props();

  let info: GetInfoRes | undefined | null = $state();
  let imageUrl = $derived(
    info ? `${apodImageUrl}?date=${data.date}` : undefined
  );
  let date: DateValue = $derived(parseDate(data.date));
  let isLoading = $state(false);
  let imageLoaded = $state(false);

  $effect(() => {
    data.apodInfo.then((res) => {
      info = res;
    });
  });

  $effect(() => {
    if (date !== parseDate(data.date)) {
      info = undefined;
      imageLoaded = false;
      goto(`/apod/${date.toDate('UTC').toISOString().split('T')[0]}`);
    }
  });

  const select = async () => {
    isLoading = true;
    let { response } = await setGoodApod({
      body: {
        date: date.toDate('UTC'),
        good: !info?.user
      }
    });
    isLoading = false;

    if (!response?.ok) {
      toast.error('Failed to update apod state');
    }
  };
</script>

<Tabs.Root value="today" class="max-h-screen grow p-4">
  <Tabs.List class="ml-10 w-fit md:ml-0">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today" class="min-h-0 grow">
    <Card.Root class="flex h-full flex-col">
      <Card.Header>
        {#if info}
          {info.title}
        {:else if info !== null}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content class="flex min-h-0 flex-1 flex-col">
        {#if info === null}
          <div class="flex min-h-0 flex-1 items-center justify-center">
            <p>No image available for today</p>
          </div>
        {:else}
          {#if info}
            <img
              class={cn(
                'min-h-0 flex-1 rounded object-contain',
                !imageLoaded && 'hidden'
              )}
              src={imageUrl}
              alt="Apod"
              onload={() => (imageLoaded = true)}
            />
          {/if}
          {#if !imageLoaded}
            <div class="min-h-0 flex-1">
              <div class="flex h-full w-full items-center justify-center">
                <Skeleton
                  class="aspect-video h-auto max-h-full w-full max-w-full"
                />
              </div>
            </div>
          {/if}
        {/if}
        <div class="mt-6 flex flex-row">
          <Datepicker
            bind:value={date}
            end={now(getLocalTimeZone())}
            class={info ? 'mr-5' : ''}
          />
          {#if info}
            <Button
              variant={info.user ? 'destructive' : 'default'}
              class="ml-auto"
              onclick={select}
              disabled={isLoading}
            >
              {#if isLoading}
                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
              {/if}
              {info.user ? 'Deselect' : 'Select'}
            </Button>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
</Tabs.Root>
