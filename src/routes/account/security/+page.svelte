<script lang="ts">
  import { AuthError } from "$lib/auth/types.svelte";
  import { special_access } from "$lib/auth/password.svelte";
  import {
    list,
    register,
    special_access as special_access_pk,
  } from "$lib/auth/passkey.svelte";
  import LoginOther from "$lib/components/form/login-other.svelte";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { LoaderCircle, KeyRound, Pencil, Trash } from "lucide-svelte";
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { interval } from "$lib/util/interval.svelte";
  import Separator from "$lib/components/ui/separator/separator.svelte";
  import { DateTime } from "luxon";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { cn } from "$lib/utils";

  let isAuthLoading = $state(false);
  let password = $state("");
  let formError = $state("");
  let passkeyError = $state("");
  let passkeyCreateError = $state("");
  let passkeyName = $state("");
  let passkeyCreateDialogOpen = $state(false);
  let passkeyCreateLoading = $state(false);

  let specialAccessWatcher = interval(() => {
    return get_token(TokenType.SpecialAccess);
  }, 1000);
  let specialAccessValid = $state(false);
  $effect(() => {
    specialAccessValid = specialAccessWatcher.value !== undefined;
  });

  let passkeys_promise = $state(list());

  const confirm = async () => {
    isAuthLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await special_access(password);

    isAuthLoading = false;

    if (ret) {
      if (ret === AuthError.Password) {
        formError = "Wrong Password";
      } else {
        formError = "There was an error while confirming access";
      }
    } else {
      password = "";
      checkAccess();
    }
  };

  const passkeyClick = async () => {
    isAuthLoading = true;
    formError = "";
    passkeyError = "";

    let ret = await special_access_pk();

    isAuthLoading = false;

    if (ret) {
      if (ret === AuthError.Passkey) {
        passkeyError = "There was an error with your passkey";
      } else {
        passkeyError = "There was an error while signing in";
      }
    } else {
      checkAccess();
    }
  };

  const checkAccess = () => {
    specialAccessValid = get_token(TokenType.SpecialAccess) !== undefined;
  };

  const createPasskey = async () => {
    if (passkeyName === "") {
      passkeyCreateError = "No Name provided";
      return;
    }

    passkeyCreateLoading = true;

    let ret = await register(passkeyName);

    passkeyCreateLoading = false;

    if (ret) {
      if (ret === AuthError.Passkey) {
        passkeyCreateError = "There was an error with your passkey";
      } else {
        passkeyCreateError = "There was an error while creating passkey";
      }
    } else {
      passkeyCreateDialogOpen = false;
      passkeys_promise = list();
    }
  };
</script>

{#if specialAccessValid}
  <div class="space-y-6">
    <div>
      <h3 class="text-xl font-medium">Security</h3>
      <p class="text-muted-foreground text-sm">Change your login settings</p>
    </div>
    <Separator />
    <div class="space-y-3">
      <h3 class="text-lg">Passkey</h3>
      <div class="border rounded-xl">
        <div class="flex items-center p-3">
          <p class="rounded-lg text-muted-foreground">Your Passkeys</p>
          <Dialog.Root bind:open={passkeyCreateDialogOpen}>
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
              <Input
                id="passkey_name"
                placeholder="Name"
                bind:value={passkeyName}
              />
              {#if passkeyCreateError !== ""}
                <span class="text-destructive truncate text-sm"
                  >{passkeyCreateError}</span
                >
              {/if}
              <Dialog.Footer>
                <Button onclick={createPasskey} disabled={passkeyCreateLoading}
                  >Create</Button
                >
              </Dialog.Footer>
            </Dialog.Content>
          </Dialog.Root>
        </div>
        <Separator />
        {#await passkeys_promise}
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
                      Created on {DateTime.fromISO(
                        passkey.created,
                      ).toLocaleString(DateTime.DATE_MED)}
                    </p>
                    <Separator orientation={"vertical"} />
                    <p class="text-muted-foreground text-sm">
                      Last used on {DateTime.fromISO(
                        passkey.used,
                      ).toLocaleString(DateTime.DATE_MED)}
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
              <div class="flex justify-center rounded-lg">
                No passkeys found
              </div>
            </div>
          {/if}
        {/await}
      </div>
    </div>
  </div>
{:else}
  <div class="flex justify-center">
    <Card.Root class="w-[350px]">
      <Card.Header>
        <Card.Title>Confirm Access</Card.Title>
        <Card.Description>Confirm access to your account</Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-6">
        <form class="grid gap-2" onsubmit={confirm}>
          <div class="grid gap-1">
            <Label class="sr-only" for="password">Password</Label>
            <Input
              id="password"
              placeholder="Password"
              type="password"
              autocapitalize="none"
              autocomplete="current-password"
              autocorrect="off"
              disabled={isAuthLoading}
              required
              bind:value={password}
            />
          </div>
          <span class="text-destructive truncate text-sm">{formError}</span>
          <Button type="submit" disabled={isAuthLoading}>
            {#if isAuthLoading}
              <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
            {/if}
            Confirm Access
          </Button>
        </form>
        <LoginOther isLoading={isAuthLoading} {passkeyError} {passkeyClick} />
      </Card.Content>
    </Card.Root>
  </div>
{/if}
