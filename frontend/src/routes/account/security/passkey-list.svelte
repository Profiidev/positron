<script lang="ts">
  import {
    Button,
    Skeleton,
    Separator,
    toast
  } from 'positron-components/components/ui';
  import {
    FormDialog,
    FormInput,
    type FormSchema
  } from 'positron-components/components/form';
  import { KeyRound, Pencil, Trash } from 'lucide-svelte';
  import type { SvelteComponent } from 'svelte';
  import { DateTime } from 'positron-components/util';
  import { RequestError } from 'positron-components/backend';
  import {
    passkey_edit_name,
    passkey_register,
    passkey_remove
  } from '$lib/backend/auth/passkey.svelte';
  import { passkey_list } from '$lib/backend/auth/stores.svelte';
  import type { SuperValidated } from 'sveltekit-superforms';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    createSchema: FormSchema<any>;
    editSchema: FormSchema<any>;
    deleteSchema: FormSchema<any>;
  }

  let { valid, requestAccess, createSchema, editSchema, deleteSchema }: Props =
    $props();

  let passkeys = $derived(passkey_list.value);
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

  const createPasskey = async (form: SuperValidated<any>) => {
    let ret = await passkey_register(form.data.name);

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return { error: 'There was an error with your passkey' };
      } else if (ret === RequestError.Conflict) {
        return { field: 'name', error: 'Name already taken' };
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
    let ret = await passkey_remove(editing);

    if (ret) {
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

  const editPasskey = async (form: SuperValidated<any>) => {
    let ret = await passkey_edit_name(form.data.name, editing);

    if (ret) {
      if (ret === RequestError.Conflict) {
        return { field: 'name', error: 'Name already taken' };
      } else {
        return { error: 'There was an error while editing passkey name' };
      }
    } else {
      toast.success('Edit successful', {
        description: `Passkey name was changed successfully from ${editing} to ${form.data.name}`
      });
    }
  };
</script>

<div class="rounded-xl border">
  <div class="flex items-center p-3">
    <p class="text-muted-foreground rounded-lg">Your Passkeys</p>
    <FormDialog
      title="Create new Passkey"
      description="Enter the name for your new passkey"
      confirm="Create"
      trigger={{
        text: 'Create new',
        variant: 'secondary',
        class: 'ml-auto',
        loadIcon: true
      }}
      onopen={startCreatePasskey}
      onsubmit={createPasskey}
      form={createSchema}
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
      form={editSchema}
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
      form={deleteSchema}
    ></FormDialog>
  </div>
  <Separator />
  {#if !passkeys}
    <div class="flex items-center p-2">
      <div class="space-y-2 p-2">
        <div class="flex items-center space-x-2">
          <Skeleton class="size-7 rounded-full" />
          <Skeleton class="h-5 w-20" />
        </div>
        <div class="flex space-x-2">
          <Skeleton class="h-4 w-36" />
          <Separator orientation={'vertical'} />
          <Skeleton class="h-4 w-40" />
        </div>
      </div>
      <Skeleton class="m-2 ml-auto size-10" />
      <Skeleton class="m-2 size-10" />
    </div>
  {:else if passkeys && passkeys.length > 0}
    {#each passkeys as passkey, i}
      {#if i > 0}
        <Separator />
      {/if}
      <div class="flex items-center p-2">
        <div class="space-y-2 p-2">
          <div class="flex space-x-2">
            <KeyRound class="size-5" />
            <h4>{passkey.name}</h4>
          </div>
          <div class="flex space-x-2">
            <p class="text-muted-foreground text-sm">
              Created on {DateTime.fromISO(passkey.created).toLocaleString(
                DateTime.DATE_MED
              )}
            </p>
            <Separator orientation={'vertical'} />
            <p class="text-muted-foreground text-sm">
              Last used on {DateTime.fromISO(passkey.used).toLocaleString(
                DateTime.DATE_MED
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
    <div class="m-5 flex justify-center rounded-lg">No passkeys found</div>
  {/if}
</div>
