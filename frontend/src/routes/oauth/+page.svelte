<script lang="ts">
  import { page } from "$app/stores";
  import Button from "$lib/components/ui/button/button.svelte";
  import { auth } from "$lib/backend/auth/oauth.svelte";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { getInfo } from "$lib/backend/account/info.svelte";
  import { goto } from "$app/navigation";
  import { AuthError, type OAuthParams } from "$lib/backend/auth/types.svelte";
  import { clear_tokens } from "$lib/backend/auth/token.svelte";
  import Avatar from "$lib/components/util/avatar.svelte";

  let isLoading = $state(false);
  let error = $state("");
  let infoData = $derived(getInfo());

  let oauth_params: OAuthParams | undefined = $derived.by(() => {
    let code = get(page).url.searchParams.get("code");
    let name = get(page).url.searchParams.get("name");

    if (code && name) {
      return {
        code,
        name,
      };
    }
  });

  let just_logged_in: string = $derived(
    get(page).url.searchParams.get("just_logged_in") || "",
  );

  const login = async (allow: boolean) => {
    if (!oauth_params) {
      error = "There was an error while login in";
      return;
    }

    error = "";
    isLoading = true;

    let ret = await auth(oauth_params, allow);

    isLoading = false;
    if (ret === AuthError.Other) {
      error = "There was an error while login in";
    } else if (ret === AuthError.Password) {
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

  const change = () => {
    clear_tokens();
    goto(`/login?code=${oauth_params?.code}&name=${oauth_params?.name}`);
  };

  onMount(() => {
    if (just_logged_in === "true") {
      login(true);
    }
  });
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
      <span class="text-destructive truncate text-sm">{error}</span>
      <div class="flex justify-between w-full">
        <Button variant="secondary" onclick={cancel}>Cancel</Button>
        <Button onclick={confirm}>Confirm</Button>
      </div>
    </Card.Footer>
  </Card.Root>
</div>
