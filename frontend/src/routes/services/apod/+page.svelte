<script lang="ts">
  import * as Tabs from 'positron-components/components/ui/tabs';
  import * as Card from 'positron-components/components/ui/card';
  import * as ScrollArea from 'positron-components/components/ui/scroll-area';
  import { Button } from 'positron-components/components/ui/button';
  import { Skeleton } from 'positron-components/components/ui/skeleton';
  import Datepicker from 'positron-components/components/util/datepicker.svelte';
  import {
    DateTime,
    getLocalTimeZone,
    now,
    parseAbsolute
  } from 'positron-components/util/time.svelte';
  import type { ApodData, ApodInfo } from '$lib/backend/services/types.svelte';
  import {
    apod,
    getApodImage,
    apod_info_list,
    getApodDate,
    setApodDate
  } from '$lib/backend/services/stores.svelte';
  import { set_good } from '$lib/backend/services/apod.svelte';
  import { LoaderCircle } from '@lucide/svelte';

  let current_data: ApodData | undefined | null = $derived(apod.value);
  let current_image: string | undefined = $derived(getApodImage());
  let apods: ApodInfo[] | undefined = $derived(apod_info_list.value);
  let dataLoading = $state(false);
  let imageLoading = $state(false);
  let isLoading = $state(false);
  let date = $state(getApodDate());
  let currentTab = $state('today');

  $effect(() => {
    dataLoading = true;
    imageLoading = true;
    setApodDate(date);
  });

  $effect(() => {
    if (current_data) {
      dataLoading = false;
      isLoading = false;
    }
  });

  $effect(() => {
    if (current_image) {
      imageLoading = false;
    }
  });

  const select = async () => {
    isLoading = true;
    await set_good(!current_data?.user, date.toDate().toISOString());
  };

  const itemClick = (data: string) => {
    date = parseAbsolute(data, getLocalTimeZone());
    currentTab = 'today';
  };
</script>

<Tabs.Root
  bind:value={currentTab}
  class="flex h-full w-full flex-col overflow-hidden p-4"
>
  <Tabs.List class="ml-10 w-fit md:ml-0">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today" class="min-h-0 flex-1">
    <Card.Root class="flex h-full flex-col">
      <Card.Header>
        {#if current_data && !dataLoading}
          {current_data.title}
        {:else if current_data !== null}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content class="flex min-h-0 flex-1 flex-col">
        {#if current_data && !imageLoading && current_image}
          <img
            class="min-h-0 flex-1 rounded object-contain"
            src={`data:image/webp;base64, ${current_image}`}
            alt="Apod"
          />
        {:else if current_data === null}
          <div class="flex min-h-0 flex-1 items-center justify-center">
            <p>No image available for today</p>
          </div>
        {:else}
          <div class="min-h-0 flex-1">
            <div class="flex h-full w-full items-center justify-center">
              <Skeleton
                class="aspect-video h-auto max-h-full w-full max-w-full"
              />
            </div>
          </div>
        {/if}
        <div class="mt-6 flex flex-row">
          <Datepicker
            bind:value={date}
            end={now(getLocalTimeZone())}
            class={current_data ? 'mr-5' : ''}
          />
          {#if current_data}
            <Button
              variant={current_data.user ? 'destructive' : 'default'}
              class="ml-auto"
              onclick={select}
              disabled={isLoading}
            >
              {#if isLoading}
                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
              {/if}
              {current_data.user ? 'Deselect' : 'Select'}
            </Button>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
  <Tabs.Content value="library" class="min-h-0 flex-1">
    <ScrollArea.ScrollArea orientation={'vertical'} class="h-full rounded">
      <div
        class="grid w-full grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-3"
      >
        {#if apods}
          {#each apods as apod}
            <div class="aspect-square w-72">
              <Button
                variant="ghost"
                class="h-full"
                onclick={() => itemClick(apod.date)}
              >
                <img
                  class="h-auto w-auto rounded object-contain"
                  src={`data:image/webp;base64, ${apod.image}`}
                  alt="Apod"
                />
              </Button>
              <p class="mt-1 ml-4">{apod.title}</p>
              <p class="text-muted-foreground ml-4">by {apod.user.name}</p>
              <p class="text-muted-foreground ml-4">
                on {DateTime.fromISO(apod.date)
                  .setLocale('de')
                  .toLocaleString(DateTime.DATE_MED)}
              </p>
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea.ScrollArea>
  </Tabs.Content>
</Tabs.Root>
