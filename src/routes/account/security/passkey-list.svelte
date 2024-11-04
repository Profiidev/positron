<script lang="ts">
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { KeyRound, Pencil, Trash } from "lucide-svelte";
  import { DateTime } from "luxon";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AuthError, type Passkey } from "$lib/auth/types.svelte";
  import { edit_name, list, register, remove } from "$lib/auth/passkey.svelte";
  import { cn } from "$lib/utils";
  import { Separator } from "$lib/components/ui/separator";

  let createError = $state("");
  let createName = $state("");
  let createDialogOpen = $state(false);
  let isLoading = $state(false);
  let passkeys: Passkey[] | undefined = $state();
  let passkeysPromise = $state(list().then((pks) => (passkeys = pks)));
  let editName = $state("");
  let editError = $state("");
  let editDialogOpen = $state(false);
  let removeDialogOpen = $state(false);
  let removeError = $state("");
  let editing = $state("");

  const createPasskey = async () => {
    if (createName === "") {
      createError = "No Name provided";
      return;
    }

    createError = "";
    isLoading = true;

    let ret = await register(createName);

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Passkey) {
        createError = "There was an error with your passkey";
      } else if (ret === AuthError.Conflict) {
        createError = "Name already taken";
      } else {
        createError = "There was an error while creating passkey";
      }
    } else {
      createName = "";
      createDialogOpen = false;
      list().then((pks) => (passkeys = pks));
    }
  };

  const startDeletePasskey = (name: string) => {
    removeError = "";
    editing = name;
    removeDialogOpen = true;
  };

  const deletePasskey = async () => {
    let ret = await remove(editing);

    if (ret) {
      removeError = "There was an error while deleting your passkey";
    } else {
      removeDialogOpen = false;
      list().then((pks) => (passkeys = pks));
    }
  };

  const startEditPasskey = (name: string) => {
    editError = "";
    editing = name;
    editDialogOpen = true;
    editName = name;
  };

  const editPasskey = async () => {
    if (editName === "") {
      editError = "No Name provided";
      return;
    }

    editError = "";
    isLoading = true;

    let ret = await edit_name(editName, editing);

    isLoading = false;

    if (ret) {
      if (ret === AuthError.Conflict) {
        editError = "Name already taken";
      } else {
        editError = "There was an error while editing passkey name";
      }
    } else {
      editDialogOpen = false;
      list().then((pks) => (passkeys = pks));
    }
  };
</script>

<div class="border rounded-xl">
  <div class="flex items-center p-3">
    <p class="rounded-lg text-muted-foreground">Your Passkeys</p>
    <Dialog.Root bind:open={createDialogOpen}>
      <Dialog.Trigger
        class={cn("ml-auto", buttonVariants({ variant: "secondary" }))}
        >Create new</Dialog.Trigger
      >
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Create new Passkey</Dialog.Title>
          <Dialog.Description
            >Enter the name for your new passkey</Dialog.Description
          >
        </Dialog.Header>
        <form onsubmit={createPasskey}>
          <Label for="passkey_name" class="sr-only">Passkey Name</Label>
          <Input id="passkey_name" placeholder="Name" bind:value={createName} />
          {#if createError !== ""}
            <span class="text-destructive truncate text-sm">{createError}</span>
          {/if}
          <Dialog.Footer class="mt-4">
            <Button type="submit" disabled={isLoading}>Create</Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
    <Dialog.Root bind:open={editDialogOpen}>
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Change Passkey Name</Dialog.Title>
          <Dialog.Description
            >Enter a new name for your passkey</Dialog.Description
          >
        </Dialog.Header>
        <form onsubmit={editPasskey}>
          <Label for="passkey_name" class="sr-only">Passkey Name</Label>
          <Input id="passkey_name" placeholder="Name" bind:value={editName} />
          {#if editError !== ""}
            <span class="text-destructive truncate text-sm">{editError}</span>
          {/if}
          <Dialog.Footer class="mt-4">
            <Button type="submit" disabled={isLoading}>Confirm</Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
    <AlertDialog.Root bind:open={removeDialogOpen}>
      <AlertDialog.Content>
        <AlertDialog.Header>
          <AlertDialog.Title
            >Do you want to delete this Passkey?</AlertDialog.Title
          >
          <AlertDialog.Description
            >This will permanently remove the "{editing}" passkey from your
            account</AlertDialog.Description
          >
        </AlertDialog.Header>
        {#if removeError !== ""}
          <span class="text-destructive truncate text-sm">{removeError}</span>
        {/if}
        <AlertDialog.Footer>
          <AlertDialog.Cancel disabled={isLoading}>Cancel</AlertDialog.Cancel>
          <AlertDialog.Action disabled={isLoading} onclick={deletePasskey}
            >Confirm</AlertDialog.Action
          >
        </AlertDialog.Footer>
      </AlertDialog.Content>
    </AlertDialog.Root>
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
    {#if passkeys}
      {#each passkeys as passkey}
        <Separator />
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
