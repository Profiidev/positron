<script lang="ts">
  import { change, info } from "$lib/auth/password.svelte";
  import { AuthError, type PasswordInfo } from "$lib/auth/types.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { DateTime } from "luxon";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let newPassword = $state("");
  let newPasswordConfirm = $state("");
  let isLoading = $state(false);
  let lastLogin: PasswordInfo | undefined = $state();

  info().then((info) => (lastLogin = info));

  const startChange = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    newPassword = "";
    newPasswordConfirm = "";
    return true;
  };

  const changeConfirm = async () => {
    if (newPassword !== newPasswordConfirm) {
      return "Passwords are not equal";
    }

    let ret = await change(newPassword, newPasswordConfirm);

    if (ret) {
      if (ret === AuthError.Password) {
        return "Passwords are not equal";
      } else {
        return "Error while updating password";
      }
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
  <FormDialog
    title="Change Password"
    description="Enter your new password below"
    confirm="Change Password"
    trigger={{
      text: "Change Password",
      variant: "secondary",
      class: "ml-auto",
    }}
    onopen={startChange}
    onsubmit={changeConfirm}
  >
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
  </FormDialog>
</div>
