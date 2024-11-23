<script lang="ts">
  import { userData } from "$lib/backend/account/info.svelte";
  import type { UserInfo } from "$lib/backend/account/types.svelte";
  import { password_change } from "$lib/backend/auth/password.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { DateTime } from "$lib/util/time.svelte";
  import { toast } from "svelte-sonner";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let newPassword = $state("");
  let newPasswordConfirm = $state("");
  let isLoading = $state(false);
  let userInfo: UserInfo | undefined = $derived(userData.value?.[0]);

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

    let ret = await password_change(newPassword, newPasswordConfirm);

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return "Passwords are not equal";
      } else {
        return "Error while updating password";
      }
    } else {
      toast.success("Update successful", {
        description: "Password was changed successfully",
      });
    }
  };
</script>

<div class="flex items-center">
  <div class="flex h-6 space-x-2 mr-2">
    {#if userInfo}
      <p class="text-muted-foreground text-sm">
        Last login {DateTime.fromISO(userInfo.last_login).toLocaleString(
          DateTime.DATE_MED,
        )}
      </p>
      <Separator orientation={"vertical"} />
      <p class="text-muted-foreground text-sm">
        Last special access {DateTime.fromISO(
          userInfo.last_special_access,
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
      loadIcon: true,
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
