<script lang="ts">
  import LoginOther from "$lib/components/form/login-other-options.svelte";
  import * as Card from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Button } from "$lib/components/ui/button";
  import { LoaderCircle } from "lucide-svelte";
  import { special_access } from "$lib/auth/password.svelte";
  import { special_access as special_access_pk } from "$lib/auth/passkey.svelte";
  import { AuthError } from "$lib/auth/types.svelte";
  import { get_token, TokenType } from "$lib/auth/token.svelte";

  interface Props {
    specialAccessValid: boolean;
  }

  let { specialAccessValid = $bindable() }: Props = $props();

  let isLoading = $state(false);
  let password = $state("");
  let formError = $state("");
  let passkeyError = $state("");

  const confirm = async () => {
    isLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await special_access(password);

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Password) {
        formError = "Wrong Password";
      } else {
        formError = "There was an error while confirming access";
      }
    } else {
      password = "";
      checkAccess();
    }
  };

  const passkeyClick = async () => {
    isLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await special_access_pk();

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Passkey) {
        passkeyError = "There was an error with your passkey";
      } else {
        passkeyError = "There was an error while signing in";
      }
    } else {
      checkAccess();
    }
  };

  const checkAccess = () => {
    specialAccessValid = get_token(TokenType.SpecialAccess) !== undefined;
  };
</script>

<div class="flex justify-center">
  <Card.Root class="w-[350px]">
    <Card.Header>
      <Card.Title>Confirm Access</Card.Title>
      <Card.Description>Confirm access to your account</Card.Description>
    </Card.Header>
    <Card.Content class="grid gap-6">
      <form class="grid gap-2" onsubmit={confirm}>
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
        <span class="text-destructive truncate text-sm">{formError}</span>
        <Button type="submit" disabled={isLoading}>
          {#if isLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {/if}
          Confirm Access
        </Button>
      </form>
      <LoginOther {isLoading} {passkeyError} {passkeyClick} />
    </Card.Content>
  </Card.Root>
</div>
