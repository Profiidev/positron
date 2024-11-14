<script lang="ts">
  import { preventDefault } from "svelte/legacy";

  import { cn } from "$lib/utils";
  import { LoaderCircle } from "lucide-svelte";
  import { Button } from "../../lib/components/ui/button/index";
  import { Input } from "../../lib/components/ui/input/index";
  import { Label } from "../../lib/components/ui/label/index";
  import { login } from "$lib/backend/auth/password.svelte";
  import { AuthError, type OAuthParams } from "$lib/backend/auth/types.svelte";
  import { goto } from "$app/navigation";
  import { confirm } from "$lib/backend/auth/totp.svelte";
  import { authenticate } from "$lib/backend/auth/passkey.svelte";
  import LoginOther from "../../lib/components/form/login-other-options.svelte";
  import Totp_6 from "$lib/components/form/totp-6.svelte";
  import { updateInfo } from "$lib/backend/account/info.svelte";

  interface Props {
    class?: string | undefined | null;
    oauth_params: OAuthParams | undefined;
  }

  let { class: className = undefined, oauth_params }: Props = $props();

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
        await login_success();
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
        await login_success();
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
      await login_success();
    }
  };

  const login_success = async () => {
    await updateInfo();
    if (oauth_params) {
      goto(
        `/oauth?code=${oauth_params.code}&name=${oauth_params.name}&just_logged_in=true`,
      );
    } else {
      goto("/");
    }
  };
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
          <Totp_6 bind:totp class="flex w-full sm:w-[350px] justify-between" />
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
