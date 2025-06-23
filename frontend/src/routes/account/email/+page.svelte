<script lang="ts">
  import {
    Separator,
    Label,
    Skeleton,
    toast
  } from 'positron-components/components/ui';
  import {
    FormDialog,
    FormInput,
    Totp_6,
    type FormType
  } from 'positron-components/components/form';
  import { RequestError } from 'positron-components/backend';
  import type { SvelteComponent } from 'svelte';
  import AccessConfirm from '../access-confirm.svelte';
  import {
    email_finish_change,
    email_start_change
  } from '$lib/backend/email.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import type { PageServerData } from './$types';
  import { confirmSchema, emailChange } from './schema.svelte';
  import { get } from 'svelte/store';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let confirmForm = {
    form: data.confirm,
    schema: confirmSchema
  };

  let changeEmailForm = {
    form: data.emailChange,
    schema: emailChange
  };

  let infoData = $derived(userData.value?.[1]);

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false)
  );

  let enteringCodes = $state(false);
  let emailFormComp: SvelteComponent | undefined = $state();

  const startChangeEmail = async () => {
    if (!specialAccessValid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    return true;
  };

  const changeEmail = async (form: FormType<any>) => {
    if (enteringCodes) {
      return enterCodes(form);
    } else {
      return enterEmail(form);
    }
  };

  const enterEmail = async (form: FormType<any>) => {
    let ret = await email_start_change(form.data.email);

    if (ret) {
      if (ret === RequestError.Conflict) {
        return { field: 'email', error: 'This email is already taken' };
      } else {
        return { error: 'There was an error while sending your emails' };
      }
    } else {
      enteringCodes = true;
      emailFormComp?.setValue({
        email_input: false
      });
      return { error: '' };
    }
  };

  const enterCodes = async (form: FormType<any>) => {
    let ret = await email_finish_change(form.data.old_code, form.data.new_code);

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return { error: 'Invalid confirm code', field: 'new_code' };
      } else {
        return { error: 'There was an error while updating your email' };
      }
    } else {
      enteringCodes = false;
      toast.success('Update successful', {
        description: 'Your email address was updated successfully'
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
        <Skeleton class="mt-1 h-5 w-48" />
      {/if}
    </div>
    <FormDialog
      bind:this={emailFormComp}
      title="Change Email"
      description={enteringCodes
        ? 'Enter the code send to your old and new email below'
        : 'Enter your new email below'}
      confirm={enteringCodes ? 'Confirm' : 'Change'}
      trigger={{
        text: 'Change Email',
        variant: 'secondary',
        class: 'ml-auto',
        loadIcon: true
      }}
      onopen={startChangeEmail}
      onsubmit={changeEmail}
      form={changeEmailForm}
    >
      {#snippet children({ props })}
        {#if !enteringCodes}
          <FormInput
            label="New Email"
            placeholder="New Email"
            key="email"
            autocapitalize="none"
            autocomplete="email"
            autocorrect="off"
            {...props}
          />
        {:else}
          <div class="space-y-4">
            <div class="space-y-2">
              <Totp_6
                label={`Code from old Email (${infoData?.email})`}
                key="old_code"
                {...props}
                class="flex justify-center"
              />
            </div>
            <div class="space-y-2">
              <Totp_6
                label={`Code from new Email (${get(props.formData.form).email})`}
                key="new_code"
                {...props}
                class="flex justify-center"
              />
            </div>
          </div>
        {/if}
      {/snippet}
    </FormDialog>
  </div>
</div>
<AccessConfirm
  bind:this={accessConfirm}
  bind:specialAccessValid
  formData={confirmForm}
/>
