<script lang="ts">
  import { preventDefault } from "svelte/legacy";

  import { cn } from "$lib/utils";
  import { LoaderCircle } from "lucide-svelte";
  import { Button } from "../../lib/components/ui/button/index";
  import { Input } from "../../lib/components/ui/input/index";
  import { Label } from "../../lib/components/ui/label/index";
  import { login } from "$lib/auth/password.svelte";
  import { AuthError } from "$lib/auth/types.svelte";
  import { goto } from "$app/navigation";
  import { confirm } from "$lib/auth/totp.svelte";
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { authenticate } from "$lib/auth/passkey.svelte";
  import LoginOther from "../../lib/components/form/login-other-options.svelte";
  import Totp_6 from "$lib/components/form/totp-6.svelte";

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
  let passkeyError = $state("");

  const onSubmit = async () => {
    if (!enterEmail) {
      isLoading = true;
      form_error = "";
      passkeyError = "";

      let ret = await confirm(totp);

      isLoading = false;

      if (ret) {
        if (ret === AuthError.Totp) {
          form_error = "Wrong TOTP Code";
        } else {
          form_error = "There was and Error while checking TOTP Code";
        }
        return;
      } else {
        goto("/");
        return;
      }
    }

    isLoading = true;
    form_error = "";
    passkeyError = "";

    let ret = await login(email, password);

    isLoading = false;

    if (typeof ret === "boolean") {
      if (ret) {
        enterEmail = false;
      } else {
        goto("/");
      }
    } else {
      if (ret === AuthError.Password) {
        form_error = "Wrong Email or Password";
      } else {
        form_error = "There was an Error while signing in";
      }
    }
  };

  const passkeyClick = async () => {
    form_error = "";
    passkeyError = "";
    isLoading = true;

    let ret = await authenticate();

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Passkey) {
        passkeyError = "There was an Error with your passkey";
      } else {
        passkeyError = "There was an Error while signing in";
      }
    } else {
      goto("/");
    }
  };

  if (get_token(TokenType.Auth)) {
    goto("/", {
      replaceState: true,
    });
  }
</script>

<div class={cn("grid gap-6", className)}>
  <form onsubmit={preventDefault(onSubmit)}>
    <div class="grid gap-2">
      {#if enterEmail}
        <div class="grid gap-1">
          <Label class="sr-only" for="email">Email</Label>
          <Input
            id="email"
            placeholder="name@example.com"
            type="email"
            autocapitalize="none"
            autocomplete="email"
            autocorrect="off"
            disabled={isLoading}
            required
            autofocus
            bind:value={email}
          />
        </div>
        <div class="grid gap-1">
          <Label class="sr-only" for="password">Password</Label>
          <Input
            id="password"
            placeholder="Password"
            type="password"
            autocapitalize="none"
            autocomplete="current-password"
            autocorrect="off"
            disabled={isLoading}
            required
            bind:value={password}
          />
        </div>
      {:else}
        <div class="grid gap-1">
          <Label class="sr-only">TOTP</Label>
          <Totp_6 bind:totp={totp} class="flex w-full sm:w-[350px] justify-between" />
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
  <LoginOther {isLoading} {passkeyError} {passkeyClick} />
</div>
