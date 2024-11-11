<script lang="ts">
  import { Separator } from "$lib/components/ui/separator";
  import type { SvelteComponent } from "svelte";
  import AccessConfirm from "../access-confirm.svelte";
  import { Label } from "$lib/components/ui/label";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { toast } from "svelte-sonner";
  import { EmailError } from "$lib/backend/email/types.svelte";
  import { finish_change, start_change } from "$lib/backend/email/manage.svelte";
  import { Input } from "$lib/components/ui/input";
  import Totp_6 from "$lib/components/form/totp-6.svelte";
  import { getInfo, updateInfo } from "$lib/backend/account/info.svelte";

  let infoData = $derived(getInfo());

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false),
  );

  let newEmail = $state("");
  let oldCode = $state("");
  let newCode = $state("");
  let enteringCodes = $state(false);

  const startChangeEmail = async () => {
    if (!specialAccessValid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    oldCode = "";
    newCode = "";
    newEmail = "";
    return true;
  };

  const changeEmail = async () => {
    if (enteringCodes) {
      return enterCodes();
    } else {
      return enterEmail();
    }
  };

  const enterEmail = async () => {
    if (newEmail === "") {
      return "No email provided";
    }

    let ret = await start_change(newEmail);

    if (ret !== null) {
      return "There was an error while sending your emails";
    } else {
      enteringCodes = true;
      return "";
    }
  };

  const enterCodes = async () => {
    if (oldCode === "" || newCode === "") {
      return "No code provided";
    }

    let ret = await finish_change(oldCode, newCode);

    if (ret) {
      if (ret === EmailError.Code) {
        return "Invalid confirm code";
      } else {
        return "There was an error while updating your email";
      }
    } else {
      await updateInfo();
      enteringCodes = false;
      toast.success("Update successful", {
        description: "Your email address was updated successfully",
      });
    }
  };
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-xl font-medium">Email</h3>
    <p class="text-muted-foreground text-sm">Change your email settings here</p>
  </div>
  <Separator />
  <div class="flex items-center">
    <div>
      <Label>Current Email</Label>
      {#if infoData}
        <p>{infoData.email}</p>
      {:else}
        <Skeleton class="h-5 w-48 mt-1" />
      {/if}
    </div>
    <FormDialog
      title="Change Email"
      description={enteringCodes
        ? "Enter the code send to your old and new email below"
        : "Enter your new email below"}
      confirm={enteringCodes ? "Confirm" : "Change"}
      trigger={{
        text: "Change Email",
        variant: "secondary",
        class: "ml-auto",
        loadIcon: true,
      }}
      onopen={startChangeEmail}
      onsubmit={changeEmail}
    >
      {#if !enteringCodes}
        <Label for="new_email" class="sr-only">New Email</Label>
        <Input
          id="new_email"
          placeholder="New Email"
          type="email"
          autocapitalize="none"
          autocomplete="email"
          autocorrect="off"
          required
          bind:value={newEmail}
        />
      {:else}
        <div class="space-y-4">
          <div class="space-y-2">
            <Label class="flex justify-center"
              >Code from old Email ({infoData?.email})</Label
            >
            <Totp_6 bind:totp={oldCode} class="flex justify-center" />
          </div>
          <div class="space-y-2">
            <Label class="flex justify-center"
              >Code from new Email ({newEmail})</Label
            >
            <Totp_6 bind:totp={newCode} class="flex justify-center" />
          </div>
        </div>
      {/if}
    </FormDialog>
  </div>
</div>
<AccessConfirm bind:this={accessConfirm} bind:specialAccessValid />
