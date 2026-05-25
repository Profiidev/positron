<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { toast } from '@profidev/pleiades/components/util/general';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import { KeyRound, Pencil, Trash } from '@lucide/svelte';
  import { type SvelteComponent } from 'svelte';
  import { DateTime } from '@profidev/pleiades/util/time.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import {
    passkeyCreateSchema,
    passkeyDeleteSchema,
    passkeyEditSchema
  } from './schema.svelte';
  import {
    editPasskeyName,
    finishRegistration,
    removePasskey,
    startRegistration,
    type PasskeyInfo
  } from '$lib/client';
  import {
    type PublicKeyCredentialCreationOptionsJSON,
    type RegistrationResponseJSON
  } from '@simplewebauthn/browser';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    passkeys?: PasskeyInfo[];
  }

  let { valid, requestAccess, passkeys }: Props = $props();

  let editing = $state('');
  let editDialog: SvelteComponent | undefined = $state();
  let deleteDialog: SvelteComponent | undefined = $state();

  const startCreatePasskey = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    return true;
  };

  const createPasskey = async (form: FormValue<typeof passkeyCreateSchema>) => {
    let { data, response } = await startRegistration();
    if (response?.status !== 200) {
      return {
        error: 'There was an error while starting passkey registration'
      };
    }

    let optionsJSON = (
      data as { publicKey: PublicKeyCredentialCreationOptionsJSON }
    ).publicKey;

    let passkeyResponse: RegistrationResponseJSON;
    try {
      const webauthnStart = (await import('@simplewebauthn/browser'))
        .startRegistration;
      passkeyResponse = await webauthnStart({ optionsJSON });
    } catch {
      return {
        error:
          'There was an error during passkey registration. Please try again.'
      };
    }

    let { response: regResponse } = await finishRegistration({
      body: {
        name: form.name,
        reg: passkeyResponse
      } as any
    });

    if (regResponse?.status !== 200) {
      if (regResponse?.status === 401) {
        return { error: 'There was an error with your passkey' };
      } else if (regResponse?.status === 409) {
        return { field: 'name', error: 'Name already taken' } as const;
      } else {
        return { error: 'There was an error while creating passkey' };
      }
    } else {
      toast.success('Creation successful', {
        description: 'Passkey was successfully added to your account'
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
    let { response } = await removePasskey({
      body: {
        name: editing
      }
    });

    if (response?.status !== 200) {
      return { error: 'There was an error while deleting your passkey' };
    } else {
      toast.success('Deletion successful', {
        description: `Passkey "${editing}" was successfully removed from your account`
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
    editDialog?.openFn();
    editDialog?.setValue({ name });
  };

  const editPasskey = async (form: FormValue<typeof passkeyEditSchema>) => {
    let { response } = await editPasskeyName({
      body: {
        old_name: editing,
        name: form.name
      }
    });

    if (response?.status !== 200) {
      if (response?.status === 401) {
        return { field: 'name', error: 'Name already taken' } as const;
      } else {
        return { error: 'There was an error while editing passkey name' };
      }
    } else {
      toast.success('Edit successful', {
        description: `Passkey name was changed successfully from ${editing} to ${form.name}`
      });
    }
  };
</script>

<div class="mt-2 flex">
  <h5 class="my-2">Passkeys:</h5>
  <FormDialog
    title="Create new Passkey"
    description="Enter the name for your new passkey"
    confirm="Create"
    trigger={{
      text: 'Add Passkey',
      class: 'ml-auto cursor-pointer',
      loadIcon: true
    }}
    onopen={startCreatePasskey}
    onsubmit={createPasskey}
    schema={passkeyCreateSchema}
  >
    {#snippet children({ props })}
      <FormInput
        label="Passkey Name"
        placeholder="Name"
        key="name"
        {...props}
      />
    {/snippet}
  </FormDialog>
  <FormDialog
    title="Change Passkey Name"
    description="Enter a new name for your passkey"
    confirm="Confirm"
    trigger={undefined}
    onsubmit={editPasskey}
    bind:this={editDialog}
    schema={passkeyEditSchema}
  >
    {#snippet children({ props })}
      <FormInput
        label="Passkey Name"
        placeholder="Name"
        key="name"
        {...props}
      />
    {/snippet}
  </FormDialog>
  <FormDialog
    title="Delete Passkey"
    description={`This will permanently remove the passkey "${editing}" from your account`}
    confirm="Confirm"
    confirmVariant="destructive"
    trigger={undefined}
    onsubmit={deletePasskey}
    bind:this={deleteDialog}
    schema={passkeyDeleteSchema}
  ></FormDialog>
</div>
<div class="rounded-xl border">
  {#if !passkeys}
    <div class="flex items-center p-1">
      <div class="space-y-1 p-2">
        <div class="flex items-center space-x-2">
          <Skeleton class="size-7 rounded-full" />
          <Skeleton class="h-5 w-20" />
        </div>
        <div class="flex space-x-1">
          <Skeleton class="h-4 w-36" />
          <Separator orientation={'vertical'} />
          <Skeleton class="h-4 w-40" />
        </div>
      </div>
      <Skeleton class="m-2 ml-auto size-8" />
      <Skeleton class="m-2 size-8" />
    </div>
  {:else if passkeys && passkeys.length > 0}
    {#each passkeys as passkey, i}
      {#if i > 0}
        <Separator />
      {/if}
      <div class="flex items-center p-1">
        <div class="space-y-1 p-2">
          <div class="flex space-x-2">
            <KeyRound class="size-5" />
            <h4>{passkey.name}</h4>
          </div>
          <div class="flex space-x-1">
            <p class="text-muted-foreground text-sm">
              Created on {DateTime?.fromISO(
                passkey.created.toString()
              ).toLocaleString(DateTime.DATE_MED)}
            </p>
            <Separator orientation={'vertical'} />
            <p class="text-muted-foreground text-sm">
              Last used on {DateTime?.fromISO(
                passkey.used.toString()
              ).toLocaleString(DateTime.DATE_MED)}
            </p>
          </div>
        </div>
        <Button
          variant="outline"
          size="icon"
          class="m-2 ml-auto cursor-pointer"
          onclick={() => startEditPasskey(passkey.name)}
        >
          <Pencil />
        </Button>
        <Button
          variant="destructive"
          size="icon"
          class="m-2 cursor-pointer"
          onclick={() => startDeletePasskey(passkey.name)}
        >
          <Trash />
        </Button>
      </div>
    {/each}
  {:else}
    <div class="m-5 flex justify-center rounded-lg">No passkeys found</div>
  {/if}
</div>
