<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { goto } from "$app/navigation";
  import Avatar from "$lib/components/util/avatar.svelte";
  import { logout, oauth_auth } from "$lib/backend/auth/other.svelte";
  import { getProfileInfo } from "$lib/backend/account/info.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import type { PageServerData } from "./$types";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_params = $derived(data.oauth_params);

  let isLoading = $state(false);
  let error = $state("");
  let infoData = $derived(getProfileInfo());

  const login = async (allow: boolean) => {
    if (!oauth_params) {
      error = "There was an error while login in";
      return;
    }

    error = "";
    isLoading = true;

    let ret = await oauth_auth(oauth_params, allow);

    isLoading = false;
    if (ret === RequestError.Other) {
      error = "There was an error while login in";
    } else if (ret === RequestError.Unauthorized) {
      error = "You are not allowed to access this Application";
    }
  };

  const confirm = () => {
    login(true);
  };

  const cancel = () => {
    login(false);
    goto("/");
  };

  const change = async () => {
    await logout();
    goto(`/login?code=${oauth_params?.code}&name=${oauth_params?.name}`);
  };
</script>

<div class="flex items-center justify-center h-full">
  <Card.Root>
    <Card.Header>
      <Card.Title>Log in to {oauth_params?.name}</Card.Title>
      <Card.Description
        >Do you want to log in to {oauth_params?.name} with the account below?</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex items-center">
      {#if infoData}
        <Avatar src={infoData.image} class="size-14" />
        <div class="grid flex-1 text-left text-sm leading-tight ml-2">
          <span class="truncate font-semibold text-lg">{infoData.name}</span>
          <span class="truncate">{infoData.email}</span>
        </div>
        <Button variant="link" onclick={change}>Change</Button>
      {:else}
        <Skeleton class="size-14 rounded-full" />
        <div class="grid flex-1 text-left text-sm leading-tight space-y-2 ml-2">
          <Skeleton class="h-5 rounded-full w-32" />
          <Skeleton class="h-3 w-32" />
        </div>
      {/if}
    </Card.Content>
    <Card.Footer class="flex flex-col">
      <span class="text-destructive truncate text-sm mb-4">{error}</span>
      <div class="flex justify-between w-full">
        <Button variant="secondary" onclick={cancel}>Cancel</Button>
        <Button onclick={confirm}>Confirm</Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
