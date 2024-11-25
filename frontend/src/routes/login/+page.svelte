<script lang="ts">
  import type { OAuthParams } from "$lib/backend/auth/types.svelte";
  import { onMount } from "svelte";
  import type { PageServerData } from "./$types";
  import Login from "./login-form.svelte";
  import { PUBLIC_IS_APP } from "$env/static/public";
  import { getTokenCookie } from "$lib/backend/cookie.svelte";
  import { goto } from "$app/navigation";
    import { browser } from "$app/environment";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_params: OAuthParams | undefined = $derived(data.oauth_params);

  onMount(() => {
    if (PUBLIC_IS_APP !== "true" || !browser) return;

    if (getTokenCookie()) {
      goto("/");
    }
  });
</script>

<div
  class="container relative h-full flex-col items-center justify-center grid lg:max-w-none lg:grid-cols-2 px-0"
>
  <div
    class="bg-muted relative hidden h-full flex-col p-10 text-white lg:flex dark:border-r"
  >
    <div class="absolute inset-0 bg-cover background-img bg-center"></div>
    <div class="relative z-20 flex items-center text-2xl">Positron</div>
  </div>
  <div class="lg:p-8">
    <div
      class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]"
    >
      <div class="flex flex-col space-y-2 text-center">
        <h1 class="text-2xl font-semibold tracking-tight">Login</h1>
        <p class="text-muted-foreground text-sm">
          Enter your login details below
        </p>
      </div>
      <Login {oauth_params} />
    </div>
  </div>
</div>

<style>
  .background-img {
    background-image: url($lib/images/login.png?enhanced);
  }
</style>
