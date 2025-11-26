<script lang="ts">
  import { Separator } from 'positron-components/components/ui/separator';
  import { Label } from 'positron-components/components/ui/label';
  import { Skeleton } from 'positron-components/components/ui/skeleton';
  import { toast } from 'positron-components/components/util/general';
  import FormDialog from 'positron-components/components/form/form-dialog.svelte';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import Totp_6 from 'positron-components/components/form/totp-6.svelte';
  import { RequestError } from 'positron-components/backend';
  import type { SvelteComponent } from 'svelte';
  import AccessConfirm from '../access-confirm.svelte';
  import {
    email_finish_change,
    email_start_change
  } from '$lib/backend/email.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import { confirmSchema, emailChange } from './schema.svelte';
  import { get } from 'svelte/store';
  import type { FormValue } from 'positron-components/components/form/types';

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

  const changeEmail = async (form: FormValue<typeof emailChange>) => {
    if (enteringCodes) {
      return enterCodes(form);
    } else {
      return enterEmail(form);
    }
  };

  const enterEmail = async (form: FormValue<typeof emailChange>) => {
    let ret = await email_start_change(form.email);

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

  const enterCodes = async (form: FormValue<typeof emailChange>) => {
    let ret = await email_finish_change(form.old_code, form.new_code);

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
      schema={emailChange}
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
  schema={confirmSchema}
/>
