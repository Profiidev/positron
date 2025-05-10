<script lang="ts">
  import type { OAuthParams } from '$lib/backend/auth/types.svelte';
  import type { PageServerData } from './$types';
  import Login from './login-form.svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { loginSchema } from './schema.svelte';
  import { test_token } from '$lib/backend/auth/other.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();
  let oauth_params: OAuthParams | undefined = $derived(data.oauth_params);

  test_token().then((valid) => {
    if (valid && browser) {
      goto('/');
    }
  });

  const loginForm = {
    form: data.loginForm,
    schema: loginSchema
  };
</script>

<div
  class="relative container grid h-full flex-col items-center justify-center px-0 lg:max-w-none lg:grid-cols-2"
>
  <div
    class="bg-muted relative hidden h-full flex-col p-10 text-white lg:flex dark:border-r"
  >
    <div class="background-img absolute inset-0 bg-cover bg-center"></div>
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
      <Login {oauth_params} {loginForm} />
    </div>
  </div>
</div>

<style>
  .background-img {
    background-image: url($lib/images/login.png?enhanced);
  }
</style>
