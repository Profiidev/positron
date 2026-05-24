<script lang="ts">
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import { emailChangeSchema } from './schema.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { confirmEmailChange, startEmailChange } from '$lib/client';
  import Totp6 from '@profidev/pleiades/components/form/totp-6.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    email: string;
  }

  let { valid, requestAccess, email }: Props = $props();

  let enteringCodes = $state(false);
  let newEmail = $state('');
  let form: FormDialog<typeof emailChangeSchema> | undefined = $state();

  const startChange = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    return true;
  };

  const changeConfirm = async (
    formData: FormValue<typeof emailChangeSchema>
  ) => {
    if (enteringCodes) {
      let { response } = await confirmEmailChange({
        body: {
          new_code: formData.new_code,
          old_code: formData.old_code
        }
      });

      if (response?.status !== 200) {
        if (response?.status === 401) {
          return {
            error: 'The code you entered is incorrect.',
            field: 'new_code'
          } as const;
        } else if (response?.status === 403) {
          return {
            error: 'The code you entered is incorrect.',
            field: 'old_code'
          } as const;
        } else {
          return {
            error: 'There was an error while confirming your email change.'
          };
        }
      } else {
        enteringCodes = false;
        toast.success('Email change successful');
      }
    } else {
      let { response } = await startEmailChange({
        body: {
          new_email: formData.email
        }
      });

      if (response?.status !== 200) {
        if (response?.status === 409) {
          return {
            error: 'This email is already in use.',
            field: 'email'
          } as const;
        } else {
          return { error: 'There was an error while sending your emails.' };
        }
      } else {
        enteringCodes = true;
        newEmail = formData.email;
        form?.setValue({
          ...formData,
          email_input: true
        });
        return { error: '' };
      }
    }
  };
</script>

<div class="mt-2 flex items-center">
  <h5>Email:</h5>
  <FormDialog
    title="Change Email"
    description="Enter your new email below"
    confirm="Change Email"
    trigger={{
      text: 'Change Email',
      variant: 'secondary',
      class: 'ml-auto cursor-pointer',
      loadIcon: true
    }}
    onopen={startChange}
    onsubmit={changeConfirm}
    schema={emailChangeSchema}
    bind:this={form}
  >
    {#snippet children({ props })}
      {#if !enteringCodes}
        <FormInput
          label="New Email"
          placeholder="mail@example.com"
          key="email"
          type="email"
          {...props}
        />
      {:else}
        <div class="space-y-4">
          <div class="space-y-2">
            <Totp6
              label={`Code from old Email (${email})`}
              key="old_code"
              {...props}
              class="flex justify-center"
            />
          </div>
          <div class="space-y-2">
            <Totp6
              label={`Code from new Email (${newEmail})`}
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
