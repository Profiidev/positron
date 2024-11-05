<script lang="ts">
  import { Separator } from "$lib/components/ui/separator";
  import { DateTime } from "luxon";
  import { Clock9 } from "lucide-svelte";
  import {
    confirm_setup,
    get_setup_code,
    info,
    is_code,
    remove,
  } from "$lib/auth/totp.svelte";
  import { AuthError, type TotpInfo } from "$lib/auth/types.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import Totp_6 from "$lib/components/form/totp-6.svelte";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let totpInfo: TotpInfo | undefined = $state();
  info().then((info) => (totpInfo = info));

  let isLoading = $state(false);
  let totpRemoveError = $state("");
  let totpRemoveOpen = $state(false);
  let totpAddError = $state("");
  let totpAddOpen = $state(false);
  let totpQr = $state("");
  let totpCode = $state("");
  let totpConfirm = $state("");

  const startRemoveTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return;
      }
    }

    totpRemoveError = "";
    totpRemoveOpen = true;
  };

  const removeTotp = async () => {
    totpRemoveError = "";
    isLoading = true;

    let ret = await remove();

    isLoading = false;

    if (ret) {
      totpRemoveError = "Error while removing TOTP";
    } else {
      totpRemoveOpen = false;
      info().then((info) => (totpInfo = info));
    }
  };

  const startAddTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return;
      }
    }

    totpAddOpen = true;
    totpAddError = "";
    totpQr = "";
    isLoading = true;

    let code = await get_setup_code();

    isLoading = false;

    if (is_code(code)) {
      totpQr = code.qr;
      totpCode = code.code;
    } else {
      totpAddError = "Error while loading QR-Code";
    }
  };

  const addTotp = async () => {
    totpAddError = "";
    isLoading = true;

    let res = await confirm_setup(totpConfirm);

    isLoading = false;

    if (res) {
      if (res === AuthError.Totp) {
        totpAddError = "TOTP code invalid";
      } else {
        totpAddError = "Error while adding TOTP";
      }
    } else {
      totpAddOpen = false;
      info().then((info) => (totpInfo = info));
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
      <Button
        class="m-2 ml-auto"
        variant="destructive"
        onclick={startRemoveTotp}>Remove</Button
      >
      <Dialog.Root bind:open={totpRemoveOpen}>
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>Remove TOTP</Dialog.Title>
            <Dialog.Description
              >Do you really want to remove the TOTP 2FA method</Dialog.Description
            >
          </Dialog.Header>
          <form onsubmit={removeTotp}>
            {#if totpRemoveError !== ""}
              <span class="text-destructive truncate text-sm"
                >{totpRemoveError}</span
              >
            {/if}
            <Dialog.Footer class="mt-4">
              <Button type="submit" variant="destructive" disabled={isLoading}
                >Remove</Button
              >
            </Dialog.Footer>
          </form>
        </Dialog.Content>
      </Dialog.Root>
    {:else}
      <Button class="m-2 ml-auto" onclick={startAddTotp}>Add</Button>
      <Dialog.Root bind:open={totpAddOpen}>
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>Add TOTP</Dialog.Title>
            <Dialog.Description
              >Scan the QR-Code below or enter the code manually and enter the
              TOTP code</Dialog.Description
            >
          </Dialog.Header>
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
          </div>
          <form onsubmit={addTotp} class="flex flex-col items-center">
            <p class="mb-2">Confirm Code</p>
            <Totp_6 bind:totp={totpConfirm} class="flex justify-center" />
            {#if totpAddError !== ""}
              <span class="text-destructive truncate text-sm"
                >{totpAddError}</span
              >
            {/if}
            <Dialog.Footer class="mt-4 ml-auto">
              <Button type="submit" disabled={isLoading}>Add</Button>
            </Dialog.Footer>
          </form>
        </Dialog.Content>
      </Dialog.Root>
    {/if}
  {:else}
    <Skeleton class="m-2 ml-auto h-10 w-20" />
  {/if}
</div>
