<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import * as Card from "$lib/components/ui/card";
  import { get_image } from "$lib/backend/services/apod.svelte";
  import type { Apod } from "$lib/backend/services/types.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AspectRatio } from "$lib/components/ui/aspect-ratio";
  import Datepicker from "$lib/components/util/datepicker.svelte";
  import { getLocalTimeZone, now, today } from "@internationalized/date";

  let current_image: Apod | undefined = $state();
  get_image(new Date().toISOString()).then((image) => (current_image = image));

  let date = $state(now(getLocalTimeZone()));
  $effect(() => {
    current_image = undefined;
    get_image(date.toDate().toISOString()).then(
      (image) => (current_image = image),
    );
  });
</script>

<Tabs.Root value="today" class="m-4">
  <Tabs.List class="ml-10 md:ml-0">
    <Tabs.Trigger value="today">Today</Tabs.Trigger>
    <Tabs.Trigger value="library">Library</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="today">
    <Card.Root>
      <Card.Header>
        {#if current_image}
          {current_image.title}
        {:else}
          <Skeleton class="h-5 w-52" />
        {/if}
      </Card.Header>
      <Card.Content>
        {#if current_image}
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
        <Datepicker
          class="mt-6"
          bind:value={date}
          end={today(getLocalTimeZone())}
        />
      </Card.Content>
    </Card.Root>
  </Tabs.Content>
  <Tabs.Content value="library"></Tabs.Content>
</Tabs.Root>
