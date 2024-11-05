<script lang="ts">
  import { change, info } from "$lib/auth/password.svelte";
  import { AuthError, type PasswordInfo } from "$lib/auth/types.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { LoaderCircle } from "lucide-svelte";
  import { DateTime } from "luxon";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let error = $state("");
  let newPassword = $state("");
  let newPasswordConfirm = $state("");
  let changeDialogOpen = $state(false);
  let isLoading = $state(false);
  let lastLogin: PasswordInfo | undefined = $state();

  info().then((info) => (lastLogin = info));

  const startChange = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return;
      }
    }

    error = "";
    newPassword = "";
    newPasswordConfirm = "";
    changeDialogOpen = true;
  };

  const changeConfirm = async () => {
    if (newPassword !== newPasswordConfirm) {
      error = "Passwords are not equal";
      return;
    }

    error = "";
    isLoading = true;

    let ret = await change(newPassword, newPasswordConfirm);

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Password) {
        error = "Passwords are not equal";
      } else {
        error = "Error while updating password";
      }
    } else {
      changeDialogOpen = false;
    }
  };
</script>

<div class="flex">
  <div class="flex h-6 space-x-2">
    {#if lastLogin}
      <p class="text-muted-foreground">
        Last login {DateTime.fromISO(lastLogin.last_login).toLocaleString(
          DateTime.DATE_MED,
        )}
      </p>
      <Separator orientation={"vertical"} />
      <p class="text-muted-foreground">
        Last special access {DateTime.fromISO(
          lastLogin.last_special_access,
        ).toLocaleString(DateTime.DATE_MED)}
      </p>
    {:else}
      <Skeleton class="h-6 w-40" />
      <Separator orientation={"vertical"} />
      <Skeleton class="h-6 w-56" />
    {/if}
  </div>
  <Button variant="secondary" class="ml-auto" onclick={startChange}>
    Change Password
  </Button>
  <Dialog.Root bind:open={changeDialogOpen}>
    <Dialog.Content>
      <Dialog.Title>Change Password</Dialog.Title>
      <Dialog.Description>Enter your new password below</Dialog.Description>
      <form class="grid gap-2" onsubmit={changeConfirm}>
        <div class="grid gap-1">
          <Label class="sr-only" for="new-password">New Password</Label>
          <Input
            id="new-password"
            placeholder="New Password"
            type="password"
            autocapitalize="none"
            autocomplete="new-password"
            autocorrect="off"
            disabled={isLoading}
            required
            bind:value={newPassword}
          />
        </div>
        <div class="grid gap-1">
          <Label class="sr-only" for="new-password-confirm"
            >Confirm New Password</Label
          >
          <Input
            id="new-password-confirm"
            placeholder="Confirm New Password"
            type="password"
            autocapitalize="none"
            autocomplete="new-password"
            autocorrect="off"
            disabled={isLoading}
            required
            bind:value={newPasswordConfirm}
          />
        </div>
        <span class="text-destructive truncate text-sm">{error}</span>
        <Dialog.Footer>
          <Button type="submit" disabled={isLoading}>
            {#if isLoading}
              <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
            {/if}
            Change Password
          </Button>
        </Dialog.Footer>
      </form>
    </Dialog.Content>
  </Dialog.Root>
</div>
