<script lang="ts">
  import { Button, Card, Skeleton } from "positron-components/components/ui";
  import { SimpleAvatar } from "positron-components/components/util";
  import { RequestError } from "positron-components/backend";
  import { goto } from "$app/navigation";
  import { logout, oauth_auth } from "$lib/backend/auth/other.svelte";
  import type { PageServerData } from "./$types";
  import { userData } from "$lib/backend/account/info.svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_params = $derived(data.oauth_params);

  let isLoading = $state(false);
  let error = $state("");
  let infoData = $derived(userData.value?.[1]);

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
        <SimpleAvatar src={infoData.image} class="size-14" />
        <div class="grid flex-1 text-left text-sm leading-tight ml-2">
          <span class="truncate font-semibold text-lg">{infoData.name}</span>
          <span class="truncate">{infoData.email}</span>
        </div>
        <Button.Button variant="link" onclick={change}>Change</Button.Button>
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
        <Button.Button variant="secondary" onclick={cancel}
          >Cancel</Button.Button
        >
        <Button.Button onclick={confirm}>Confirm</Button.Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
