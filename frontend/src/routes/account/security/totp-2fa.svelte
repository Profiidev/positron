<script lang="ts">
  import { Separator } from "$lib/components/ui/separator";
  import { Clock9 } from "lucide-svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Badge } from "$lib/components/ui/badge";
  import Totp_6 from "$lib/components/form/totp-6.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { toast } from "svelte-sonner";
  import { DateTime } from "$lib/util/time.svelte";
  import type { UserInfo } from "$lib/backend/account/types.svelte";
  import {
    is_code,
    totp_confirm_setup,
    totp_get_setup_code,
    totp_remove,
  } from "$lib/backend/auth/totp.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import { userData } from "$lib/backend/account/info.svelte";
  import type { FormSchema } from "$lib/components/form/form.svelte";
  import type { SuperValidated } from "sveltekit-superforms";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    removeForm: FormSchema<any>;
    addForm: FormSchema<any>;
  }

  let { valid, requestAccess, addForm, removeForm }: Props = $props();

  let userInfo: UserInfo | undefined = $derived(userData.value?.[0]);

  let totpQr = $state("");
  let totpCode = $state("");

  const startRemoveTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }
    return true;
  };

  const removeTotp = async () => {
    let ret = await totp_remove();

    if (ret) {
      return { error: "Error while removing TOTP" };
    } else {
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
      let code = await totp_get_setup_code();

      if (is_code(code)) {
        totpQr = code.qr;
        totpCode = code.code;
      }
    };

    fetch();
    return true;
  };

  const addTotp = async (form: SuperValidated<any>) => {
    let res = await totp_confirm_setup(form.data.code);

    if (res) {
      if (res === RequestError.Unauthorized) {
        return { error: "TOTP code invalid", field: "code" };
      } else {
        return { error: "Error while adding TOTP" };
      }
    } else {
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
      {#if userInfo}
        {#if userInfo.totp_enabled}
          <Badge>Enabled</Badge>
        {:else}
          <Badge variant="destructive">Disabled</Badge>
        {/if}
      {:else}
        <Skeleton class="h-6 w-16 rounded-full" />
      {/if}
    </div>
    <div class="flex space-x-2">
      {#if userInfo}
        <p class="text-muted-foreground text-sm">
          Created on {userInfo.totp_enabled
            ? DateTime.fromISO(userInfo.totp_created!).toLocaleString(
                DateTime.DATE_MED,
              )
            : "-"}
        </p>
        <Separator orientation={"vertical"} />
        <p class="text-muted-foreground text-sm">
          Last used on {userInfo.totp_enabled
            ? DateTime.fromISO(userInfo.totp_last_used!).toLocaleString(
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
  {#if userInfo}
    {#if userInfo.totp_enabled}
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
        form={removeForm}
      ></FormDialog>
    {:else}
      <FormDialog
        title="Add TOTP"
        description="Scan the QR-Code below or enter the code manually and enter the TOTP code"
        confirm="Add"
        trigger={{ text: "Add", class: "m-2 ml-auto", loadIcon: true }}
        onopen={startAddTotp}
        onsubmit={addTotp}
        form={addForm}
      >
        {#snippet children({ props })}
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
            <Totp_6
              label="Confirm Code"
              key="code"
              {...props}
              class="flex justify-center"
            />
          </div>
        {/snippet}
      </FormDialog>
    {/if}
  {:else}
    <Skeleton class="m-2 ml-auto h-10 w-20" />
  {/if}
</div>
