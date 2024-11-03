<script lang="ts">
  import { preventDefault } from 'svelte/legacy';

  import { cn } from "$lib/utils";
  import { Key, LoaderCircle } from "lucide-svelte";
  import { Button } from "../ui/button/index";
  import { Input } from "../ui/input/index";
  import { Label } from "../ui/label/index";
  import { fetch_key, login } from '$lib/auth/password.svelte';
  import { AuthError } from '$lib/auth/error.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { confirm } from '$lib/auth/totp.svelte';
  import { get_token, TokenType } from '$lib/auth/token.svelte';
  import { InputOTP } from '../ui/input-otp';
  import InputOtpGroup from '../ui/input-otp/input-otp-group.svelte';
  import InputOtpSlot from '../ui/input-otp/input-otp-slot.svelte';
  import InputOtpSeparator from '../ui/input-otp/input-otp-separator.svelte';
  import { authenticate } from '$lib/auth/passkey.svelte';

  interface Props {
    class?: string | undefined | null;
  }

  let { class: className = undefined }: Props = $props();

  let enterEmail = $state(true);
  let isLoading = $state(false);
  let email = $state("");
  let password = $state("");
  let totp = $state("");
  let form_error = $state("");
  let passkey_error = $state("");

  const reset = () => {
    enterEmail = true;
    isLoading = false;
    email = "";
    password = "";
    form_error = "";
  }

  const onSubmit = async () => {
    if(!enterEmail) {
      isLoading = true;
      form_error = "";
      passkey_error = "";

      let ret = await confirm(totp);

      isLoading = false;

      if(ret) {
        if(ret === AuthError.Totp) {
          form_error = "Wrong TOTP Code";
        } else {
          form_error = "There was and Error while checking TOTP Code";
        }
      } else {
        goto("/");
        return;
      }
    }

    isLoading = true;
    form_error = "";
    passkey_error = "";

    let ret = await login(email, password);

    isLoading = false;

    if (typeof ret === "boolean") {
      if(ret) {
        enterEmail = false;
      } else {
        goto("/");
      }
    } else {
      if(ret === AuthError.Password) {
        form_error = "Wrong Email or Password";
      } else {
        form_error = "There was an Error while signing in";
      }
    }
  }

  const passkey = async () => {
    form_error = "";
    passkey_error = "";

    let ret = await authenticate();
    if(ret) {
      if(ret === AuthError.Passkey) {
        passkey_error = "There was an Error with your passkey";
      } else {
        passkey_error = "There was an Error while signing in";
      }
    } else {
      goto("/");
    }
  }

  onMount(async () => {
    await fetch_key();
  });

  if(get_token(TokenType.Auth)) {
    goto("/");
  }
</script>

<div class={cn("grid gap-6", className)}>
  <form onsubmit={preventDefault(onSubmit)}>
    <div class="grid gap-2">
      {#if enterEmail}
        <div class="grid gap-1">
          <Label class="sr-only" for="email">Email</Label>
          <Input id="email" placeholder="name@example.com" type="email" autocapitalize="none" autocomplete="email" autocorrect="off" disabled={isLoading} required autofocus bind:value={email} />
        </div>
        <div class="grid gap-1">
          <Label class="sr-only" for="password">Password</Label>
          <Input id="password" placeholder="Password" type="password" autocapitalize="none" autocomplete="current-password" autocorrect="off" disabled={isLoading} required bind:value={password} />
        </div>
      {:else}
        <div class="grid gap-1">
          <Label class="sr-only" for="email">TOTP</Label>
          <InputOTP maxlength={6} bind:value={totp} class="flex w-full sm:w-[350px] justify-between" required autofocus>
            {#snippet children({ cells })}
              <InputOtpGroup>
                {#each cells.slice(0, 3) as cell}
                  <InputOtpSlot {cell} />
                {/each}
              </InputOtpGroup>
              <InputOtpSeparator />
              <InputOtpGroup>
                {#each cells.slice(3, 6) as cell}
                  <InputOtpSlot {cell} />
                {/each}
              </InputOtpGroup>
            {/snippet}
          </InputOTP>
        </div>
      {/if}
      <span class="text-destructive truncate text-sm">{form_error}</span>
      <Button type="submit" disabled={isLoading}>
        {#if isLoading}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {#if enterEmail}
          Sign In
        {:else}
          Confirm
        {/if}
      </Button>
    </div>
  </form>
  <div class="relative">
    <div class="absolute inset-0 flex items-center">
      <span class="w-full border-t"></span>
    </div>
    <div class="relative flex justify-center text-xs uppercase">
      <span class="bg-background text-muted-foreground px-2">Or continue with </span>
    </div>
  </div>
  {#if passkey_error !== ""}
    <span class="text-destructive truncate text-sm">{passkey_error}</span>
  {/if}
  <Button variant="outline" type="button" disabled={isLoading} onclick={passkey}>
    {#if isLoading}
      <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
    {:else}
      <Key class="mr-2 h-4 w-4" />
    {/if}
    Passkey
  </Button>
</div>
