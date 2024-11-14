<script lang="ts">
  import { Separator } from "$lib/components/ui/separator";
  import { Clock9 } from "lucide-svelte";
  import {
    confirm_setup,
    get_setup_code,
    info,
    is_code,
    remove,
  } from "$lib/backend/auth/totp.svelte";
  import { AuthError, type TotpInfo } from "$lib/backend/auth/types.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Badge } from "$lib/components/ui/badge";
  import Totp_6 from "$lib/components/form/totp-6.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { toast } from "svelte-sonner";
  import { DateTime } from "$lib/util/time.svelte";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let totpInfo: TotpInfo | undefined = $state();
  info().then((info) => (totpInfo = info));

  let totpQr = $state("");
  let totpCode = $state("");
  let totpConfirm = $state("");

  const startRemoveTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }
    return true;
  };

  const removeTotp = async () => {
    let ret = await remove();

    if (ret) {
      return "Error while removing TOTP";
    } else {
      info().then((info) => (totpInfo = info));
      toast.success("Remove successful", {
        description: "TOTP was removed successfully from your account",
      });
    }
  };

  const startAddTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    totpQr = "";

    const fetch = async () => {
      let code = await get_setup_code();

      if (is_code(code)) {
        totpQr = code.qr;
        totpCode = code.code;
      }
    };

    fetch();
    return true;
  };

  const addTotp = async () => {
    let res = await confirm_setup(totpConfirm);

    if (res) {
      if (res === AuthError.Totp) {
        return "TOTP code invalid";
      } else {
        return "Error while adding TOTP";
      }
    } else {
      info().then((info) => (totpInfo = info));
      toast.success("Addition successful", {
        description: "TOTP was added successfully to your account",
      });
    }
  };
</script>

<div class="flex items-center">
  <div class="flex flex-col space-y-2 p-2">
    <div class="flex space-x-2">
      <Clock9 class="size-5" />
      <h4>TOTP</h4>
      {#if totpInfo}
        {#if totpInfo.enabled}
          <Badge>Enabled</Badge>
        {:else}
          <Badge variant="destructive">Disabled</Badge>
        {/if}
      {:else}
        <Skeleton class="h-6 w-16 rounded-full" />
      {/if}
    </div>
    <div class="flex space-x-2">
      {#if totpInfo}
        <p class="text-muted-foreground text-sm">
          Created on {totpInfo.enabled
            ? DateTime.fromISO(totpInfo.created!).toLocaleString(
                DateTime.DATE_MED,
              )
            : "-"}
        </p>
        <Separator orientation={"vertical"} />
        <p class="text-muted-foreground text-sm">
          Last used on {totpInfo.enabled
            ? DateTime.fromISO(totpInfo.last_used!).toLocaleString(
                DateTime.DATE_MED,
              )
            : "-"}
        </p>
      {:else}
        <div class="flex space-x-2">
          <Skeleton class="h-4 w-36" />
          <Separator orientation={"vertical"} />
          <Skeleton class="h-4 w-40" />
        </div>
      {/if}
    </div>
  </div>
  {#if totpInfo}
    {#if totpInfo.enabled}
      <FormDialog
        title="Remove TOTP"
        description="Do you really want to remove the TOTP 2FA method"
        confirm="Remove"
        confirmVariant="destructive"
        trigger={{
          text: "Remove",
          variant: "destructive",
          class: "m-2 ml-auto",
          loadIcon: true,
        }}
        onopen={startRemoveTotp}
        onsubmit={removeTotp}
      ></FormDialog>
    {:else}
      <FormDialog
        title="Add TOTP"
        description="Scan the QR-Code below or enter the code manually and enter the TOTP code"
        confirm="Add"
        trigger={{ text: "Add", class: "m-2 ml-auto", loadIcon: true }}
        onopen={startAddTotp}
        onsubmit={addTotp}
      >
        <div class="flex items-center flex-col space-y-2">
          {#if totpQr !== ""}
            <img
              class="size-60"
              src={`data:image/png;base64, ${totpQr}`}
              alt="QR"
            />
            <p class="text-muted-foreground">Or use the code</p>
            <p class="bg-muted px-1 rounded">{totpCode}</p>
          {:else}
            <Skeleton class="size-60" />
            <p class="text-muted-foreground">Or use the code</p>
            <Skeleton class="h-6 w-80" />
          {/if}
          <p class="mb-2">Confirm Code</p>
          <Totp_6 bind:totp={totpConfirm} class="flex justify-center" />
        </div>
      </FormDialog>
    {/if}
  {:else}
    <Skeleton class="m-2 ml-auto h-10 w-20" />
  {/if}
</div>
