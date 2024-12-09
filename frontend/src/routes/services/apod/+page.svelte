<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import * as Card from "$lib/components/ui/card";
  import type { Apod } from "$lib/backend/services/types.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AspectRatio } from "$lib/components/ui/aspect-ratio";
  import Datepicker from "$lib/components/util/datepicker.svelte";
  import { getLocalTimeZone, now } from "@internationalized/date";
  import {
    apod,
    getApodDate,
    setApodDate,
  } from "$lib/backend/services/stores.svelte";
  import { Button } from "$lib/components/ui/button";
  import { set_good } from "$lib/backend/services/apod.svelte";
    import { LoaderCircle } from "lucide-svelte";

  let current_image: Apod | undefined = $derived(apod.value);
  let imageLoading = $state(false);
  let isLoading = $state(false);
  let date = $state(getApodDate());

  $effect(() => {
    imageLoading = true;
    setApodDate(date);
  });

  $effect(() => {
    if (current_image) {
      imageLoading = false;
      isLoading = false;
    }
  });

  const select = async () => {
    isLoading = true;
    await set_good(!current_image?.user, date.toDate().toISOString());
  };
</script>

<Tabs.Root value="today" class="m-4">
  <Tabs.List class="ml-10 md:ml-0">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today">
    <Card.Root>
      <Card.Header>
        {#if current_image && !imageLoading}
          {current_image.title}
        {:else}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content>
        {#if current_image && !imageLoading}
          <img
            class="rounded"
            src={`data:image/webp;base64, ${current_image.image}`}
            alt="Apod"
          />
        {:else}
          <AspectRatio ratio={16 / 9}>
            <Skeleton class="w-full h-full" />
          </AspectRatio>
        {/if}
        <div class="flex flex-row mt-6">
          <Datepicker bind:value={date} end={now(getLocalTimeZone())} />
          {#if current_image}
            <Button
              variant={current_image.user ? "destructive" : "default"}
              class="ml-auto"
              onclick={select}
              disabled={isLoading}
            >
              {#if isLoading}
                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
              {/if}
              {current_image.user ? "Deselect" : "Select"}
            </Button>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
  <Tabs.Content value="library"></Tabs.Content>
</Tabs.Root>
