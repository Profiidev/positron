<script lang="ts">
  import {
    Tabs,
    Card,
    Skeleton,
    Button,
    ScrollArea,
  } from "positron-components/components/ui";
  import { Datepicker } from "positron-components/components/util";
  import { DateTime } from "positron-components/util";
  import type { ApodData, ApodInfo } from "$lib/backend/services/types.svelte";
  import {
    getLocalTimeZone,
    now,
    parseAbsolute,
  } from "@internationalized/date";
  import {
    apod,
    getApodImage,
    apod_info_list,
    getApodDate,
    setApodDate,
  } from "$lib/backend/services/stores.svelte";
  import { set_good } from "$lib/backend/services/apod.svelte";
  import { LoaderCircle } from "lucide-svelte";

  let current_data: ApodData | undefined | null = $derived(apod.value);
  let current_image: string | undefined = $derived(getApodImage());
  let apods: ApodInfo[] | undefined = $derived(apod_info_list.value);
  let dataLoading = $state(false);
  let imageLoading = $state(false);
  let isLoading = $state(false);
  let date = $state(getApodDate());
  let currentTab = $state("today");

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
    currentTab = "today";
  };
</script>

<Tabs.Root
  bind:value={currentTab}
  class="p-4 h-full flex flex-col overflow-hidden w-full"
>
  <Tabs.List class="ml-10 md:ml-0 w-fit">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today" class="flex-1 min-h-0">
    <Card.Root class="h-full flex flex-col">
      <Card.Header>
        {#if current_data && !dataLoading}
          {current_data.title}
        {:else if current_data !== null}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content class="flex flex-col flex-1 min-h-0">
        {#if current_data && !imageLoading && current_image}
          <img
            class="rounded flex-1 min-h-0 object-contain"
            src={`data:image/webp;base64, ${current_image}`}
            alt="Apod"
          />
        {:else if current_data === null}
          <div class="flex-1 min-h-0 flex justify-center items-center">
            <p>No image available for today</p>
          </div>
        {:else}
          <div class="flex-1 min-h-0">
            <div class="w-full h-full flex justify-center items-center">
              <Skeleton
                class="w-full h-auto aspect-video max-w-full max-h-full"
              />
            </div>
          </div>
        {/if}
        <div class="flex flex-row mt-6">
          <Datepicker
            bind:value={date}
            end={now(getLocalTimeZone())}
            class={current_data ? "mr-5" : ""}
          />
          {#if current_data}
            <Button.Button
              variant={current_data.user ? "destructive" : "default"}
              class="ml-auto"
              onclick={select}
              disabled={isLoading}
            >
              {#if isLoading}
                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
              {/if}
              {current_data.user ? "Deselect" : "Select"}
            </Button.Button>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
  <Tabs.Content value="library" class="flex-1 min-h-0">
    <ScrollArea.ScrollArea orientation={"vertical"} class="h-full rounded">
      <div
        class="grid w-full gap-3 grid-cols-[repeat(auto-fill,minmax(18rem,1fr))]"
      >
        {#if apods}
          {#each apods as apod}
            <div class="w-72 aspect-square">
              <Button.Button
                variant="ghost"
                class="h-full"
                onclick={() => itemClick(apod.date)}
              >
                <img
                  class="rounded object-contain h-auto w-auto"
                  src={`data:image/webp;base64, ${apod.image}`}
                  alt="Apod"
                />
              </Button.Button>
              <p class="mt-1 ml-4">{apod.title}</p>
              <p class="text-muted-foreground ml-4">by {apod.user.name}</p>
              <p class="text-muted-foreground ml-4">
                on {DateTime.fromISO(apod.date)
                  .setLocale("de")
                  .toLocaleString(DateTime.DATE_MED)}
              </p>
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea.ScrollArea>
  </Tabs.Content>
</Tabs.Root>
