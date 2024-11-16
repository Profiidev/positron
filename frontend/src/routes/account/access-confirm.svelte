<script lang="ts">
  import LoginOther from "$lib/components/form/login-other-options.svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Button } from "$lib/components/ui/button";
  import { LoaderCircle } from "lucide-svelte";
  import { interval } from "$lib/util/interval.svelte";
  import { password_special_access } from "$lib/backend/auth/password.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import { passkey_special_access } from "$lib/backend/auth/passkey.svelte";
  import { browser } from "$app/environment";

  interface Props {
    specialAccessValid: boolean;
  }

  let { specialAccessValid = $bindable(false) }: Props = $props();

  let specialAccessWatcher = interval(() => {
    if (!browser) {
      return;
    }

    let match = document.cookie.match(
      new RegExp("(^| )" + "special_valid" + "=([^;]+)"),
    );
    if (match) return Boolean(match[2]);
  }, 1000);
  $effect(() => {
    specialAccessValid =
      specialAccessWatcher.value !== undefined && specialAccessWatcher.value;
  });

  let cb: (value: boolean) => void;
  let open = $state(false);
  let isLoading = $state(false);
  let password = $state("");
  let formError = $state("");
  let passkeyError = $state("");

  export const requestAccess = async () => {
    return new Promise<boolean>((resolve) => {
      cb = resolve;
      open = true;
    });
  };

  const confirm = async () => {
    isLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await password_special_access(password);

    isLoading = false;

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        formError = "Wrong Password";
      } else {
        formError = "There was an error while confirming access";
      }
    } else {
      password = "";
      cb(true);
      open = false;
    }
  };

  const passkeyClick = async () => {
    isLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await passkey_special_access();

    isLoading = false;

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        passkeyError = "There was an error with your passkey";
      } else {
        passkeyError = "There was an error while signing in";
      }
    } else {
      cb(true);
      open = false;
    }
  };

  const onOpenChange = (open: boolean) => {
    if (!open) {
      cb(false);
    }
  };
</script>

<Dialog.Root {onOpenChange} bind:open>
  <Dialog.Content class="grid gap-6 w-[350px]">
    <Dialog.Header>
      <Dialog.Title>Confirm Access</Dialog.Title>
      <Dialog.Description>Confirm access to your account</Dialog.Description>
    </Dialog.Header>
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
  </Dialog.Content>
</Dialog.Root>
