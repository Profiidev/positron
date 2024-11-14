<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { KeyRound, Pencil, Trash } from "lucide-svelte";
  import { DateTime } from "luxon";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AuthError, type Passkey } from "$lib/backend/auth/types.svelte";
  import {
    edit_name,
    list,
    register,
    remove,
  } from "$lib/backend/auth/passkey.svelte";
  import { Separator } from "$lib/components/ui/separator";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import type { SvelteComponent } from "svelte";
  import { toast } from "svelte-sonner";

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  let createName = $state("");
  let passkeys: Passkey[] | undefined = $state();
  let passkeysPromise = $state(list().then((pks) => (passkeys = pks)));
  let editName = $state("");
  let editing = $state("");
  let editDialog: SvelteComponent | undefined = $state();
  let deleteDialog: SvelteComponent | undefined = $state();

  const startCreatePasskey = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    createName = "";
    return true;
  };

  const createPasskey = async () => {
    if (createName === "") {
      return "No Name provided";
    }

    let ret = await register(createName);

    if (ret) {
      if (ret === AuthError.Passkey) {
        return "There was an error with your passkey";
      } else if (ret === AuthError.Conflict) {
        return "Name already taken";
      } else {
        return "There was an error while creating passkey";
      }
    } else {
      createName = "";
      list().then((pks) => (passkeys = pks));
      toast.success("Creation successful", {
        description: "Passkey was successfully added to your account",
      });
    }
  };

  const startDeletePasskey = async (name: string) => {
    if (!valid) {
      if (!(await requestAccess())) {
        return;
      }
    }

    editing = name;
    deleteDialog?.openFn();
  };

  const deletePasskey = async () => {
    let ret = await remove(editing);

    if (ret) {
      return "There was an error while deleting your passkey";
    } else {
      list().then((pks) => (passkeys = pks));
      toast.success("Deletion successful", {
        description: `Passkey "${editing}" was successfully removed from your account`,
      });
    }
  };

  const startEditPasskey = async (name: string) => {
    if (!valid) {
      if (!(await requestAccess())) {
        return;
      }
    }

    editing = name;
    editName = name;
    editDialog?.openFn();
  };

  const editPasskey = async () => {
    if (editName === "") {
      return "No Name provided";
    }

    let ret = await edit_name(editName, editing);

    if (ret) {
      if (ret === AuthError.Conflict) {
        return "Name already taken";
      } else {
        return "There was an error while editing passkey name";
      }
    } else {
      list().then((pks) => (passkeys = pks));
      toast.success("Edit successful", {
        description: `Passkey name was changed successfully from ${editing} to ${editName}`,
      });
    }
  };
</script>

<div class="border rounded-xl">
  <div class="flex items-center p-3">
    <p class="rounded-lg text-muted-foreground">Your Passkeys</p>
    <FormDialog
      title="Create new Passkey"
      description="Enter the name for your new passkey"
      confirm="Create"
      trigger={{
        text: "Create new",
        variant: "secondary",
        class: "ml-auto",
        loadIcon: true,
      }}
      onopen={startCreatePasskey}
      onsubmit={createPasskey}
    >
      <Label for="passkey_name" class="sr-only">Passkey Name</Label>
      <Input
        id="passkey_name"
        placeholder="Name"
        required
        bind:value={createName}
      />
    </FormDialog>
    <FormDialog
      title="Change Passkey Name"
      description="Enter a new name for your passkey"
      confirm="Confirm"
      trigger={undefined}
      onsubmit={editPasskey}
      bind:this={editDialog}
    >
      <Label for="passkey_name" class="sr-only">Passkey Name</Label>
      <Input
        id="passkey_name"
        placeholder="Name"
        required
        bind:value={editName}
      />
    </FormDialog>
    <FormDialog
      title="Delete Passkey"
      description={`This will permanently remove the passkey "${editing}" from your account`}
      confirm="Confirm"
      confirmVariant="destructive"
      trigger={undefined}
      onsubmit={deletePasskey}
      bind:this={deleteDialog}
    ></FormDialog>
  </div>
  <Separator />
  {#await passkeysPromise}
    <div class="flex p-2 items-center">
      <div class="space-y-2 p-2">
        <div class="flex space-x-2 items-center">
          <Skeleton class="size-7 rounded-full" />
          <Skeleton class="h-5 w-20" />
        </div>
        <div class="flex space-x-2">
          <Skeleton class="h-4 w-36" />
          <Separator orientation={"vertical"} />
          <Skeleton class="h-4 w-40" />
        </div>
      </div>
      <Skeleton class="m-2 ml-auto size-10" />
      <Skeleton class="m-2 size-10" />
    </div>
  {:then}
    {#if passkeys && passkeys.length > 0}
      {#each passkeys as passkey, i}
        {#if i > 0}
          <Separator />
        {/if}
        <div class="flex p-2 items-center">
          <div class="space-y-2 p-2">
            <div class="flex space-x-2">
              <KeyRound class="size-5" />
              <h4>{passkey.name}</h4>
            </div>
            <div class="flex space-x-2">
              <p class="text-muted-foreground text-sm">
                Created on {DateTime.fromISO(passkey.created).toLocaleString(
                  DateTime.DATE_MED,
                )}
              </p>
              <Separator orientation={"vertical"} />
              <p class="text-muted-foreground text-sm">
                Last used on {DateTime.fromISO(passkey.used).toLocaleString(
                  DateTime.DATE_MED,
                )}
              </p>
            </div>
          </div>
          <Button
            variant="outline"
            size="icon"
            class="m-2 ml-auto"
            onclick={() => startEditPasskey(passkey.name)}
          >
            <Pencil />
          </Button>
          <Button
            variant="destructive"
            size="icon"
            class="m-2"
            onclick={() => startDeletePasskey(passkey.name)}
          >
            <Trash />
          </Button>
        </div>
      {/each}
    {:else}
      <div class="flex justify-center rounded-lg m-5">No passkeys found</div>
    {/if}
  {/await}
</div>
