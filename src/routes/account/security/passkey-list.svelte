<script lang="ts">
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { KeyRound, Pencil, Trash } from "lucide-svelte";
  import { DateTime } from "luxon";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { AuthError } from "$lib/auth/types.svelte";
  import { list, register } from "$lib/auth/passkey.svelte";
  import { cn } from "$lib/utils";
  import { Separator } from "$lib/components/ui/separator";

  let createError = $state("");
  let name = $state("");
  let dialogOpen = $state(false);
  let isLoading = $state(false);
  let passkeysPromise = $state(list());

  const createPasskey = async () => {
    if (name === "") {
      createError = "No Name provided";
      return;
    }

    isLoading = true;

    let ret = await register(name);

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
      dialogOpen = false;
      passkeysPromise = list();
    }
  };
</script>

<div class="border rounded-xl">
  <div class="flex items-center p-3">
    <p class="rounded-lg text-muted-foreground">Your Passkeys</p>
    <Dialog.Root bind:open={dialogOpen}>
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
        <Label for="passkey_name" class="sr-only">Passkey Name</Label>
        <Input id="passkey_name" placeholder="Name" bind:value={name} />
        {#if createError !== ""}
          <span class="text-destructive truncate text-sm"
            >{createError}</span
          >
        {/if}
        <Dialog.Footer>
          <Button onclick={createPasskey} disabled={isLoading}
            >Create</Button
          >
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>
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
  {:then passkeys}
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
          <Button variant="outline" size="icon" class="m-2 ml-auto">
            <Pencil />
          </Button>
          <Button variant="destructive" size="icon" class="m-2">
            <Trash />
          </Button>
        </div>
      {/each}
    {:else}
      <div class="hover:bg-transparent">
        <div class="flex justify-center rounded-lg">No passkeys found</div>
      </div>
    {/if}
  {/await}
</div>
