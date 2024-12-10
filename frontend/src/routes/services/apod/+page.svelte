<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import * as Card from "$lib/components/ui/card";
  import type { ApodData, ApodInfo } from "$lib/backend/services/types.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AspectRatio } from "$lib/components/ui/aspect-ratio";
  import Datepicker from "$lib/components/util/datepicker.svelte";
  import { getLocalTimeZone, now } from "@internationalized/date";
  import {
    apod,
    getApodImage,
    apod_info_list,
    getApodDate,
    setApodDate,
  } from "$lib/backend/services/stores.svelte";
  import { Button } from "$lib/components/ui/button";
  import { set_good } from "$lib/backend/services/apod.svelte";
  import { LoaderCircle } from "lucide-svelte";
  import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";

  let current_data: ApodData | undefined = $derived(apod.value);
  let current_image: string | undefined = $derived(getApodImage());
  let apods: ApodInfo[] | undefined = $derived(apod_info_list.value);
  let dataLoading = $state(false);
  let imageLoading = $state(false);
  let isLoading = $state(false);
  let date = $state(getApodDate());

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
</script>

<Tabs.Root value="today" class="p-4 h-full flex flex-col overflow-hidden">
  <Tabs.List class="ml-10 md:ml-0 w-fit">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today" class="flex-1 min-h-0">
    <Card.Root class="h-full flex flex-col">
      <Card.Header>
        {#if current_data && !dataLoading}
          {current_data.title}
        {:else}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content class="flex flex-col flex-1 min-h-0">
        {#if current_data && !imageLoading && current_image}
          <img
            class="rounded flex-1 min-h-0 w-auto"
            src={`data:image/webp;base64, ${current_image}`}
            alt="Apod"
          />
        {:else}
          <AspectRatio ratio={16 / 9} class="flex-1 min-h-0">
            <Skeleton class="w-full h-full" />
          </AspectRatio>
        {/if}
        <div class="flex flex-row mt-6">
          <Datepicker bind:value={date} end={now(getLocalTimeZone())} />
          {#if current_data}
            <Button
              variant={current_data.user ? "destructive" : "default"}
              class="ml-auto"
              onclick={select}
              disabled={isLoading}
            >
              {#if isLoading}
                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
              {/if}
              {current_data.user ? "Deselect" : "Select"}
            </Button>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
  <Tabs.Content value="library" class="flex-1 min-h-0">
    <ScrollArea orientation={"vertical"} class="h-full rounded">
      <div
        class="grid w-full gap-3 grid-cols-[repeat(auto-fit,minmax(18rem,1fr))]"
      >
        {#if apods}
          {#each apods as apod}
            <div class="w-72 aspect-square">
              <img
                class="rounded object-contain h-auto w-auto"
                src={`data:image/webp;base64, ${apod.image}`}
                alt="Apod"
              />
              <p class="mt-1">{apod.title}</p>
              <p class="text-muted-foreground">by {apod.user.name}</p>
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea>
  </Tabs.Content>
</Tabs.Root>
