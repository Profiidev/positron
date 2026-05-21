<script lang="ts">
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { passwordChange } from './schema.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { getEncrypt } from '$lib/backend/auth.svelte';
  import { changePassword } from '$lib/client';
  import { Label } from '@profidev/pleiades/components/ui/label';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
  }

  let { valid, requestAccess }: Props = $props();

  const startChange = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    return true;
  };

  const changeConfirm = async (form: FormValue<typeof passwordChange>) => {
    if (form.password !== form.password_confirm) {
      return { error: 'Passwords are not equal', field: 'password_confirm' };
    }

    let encrypt = getEncrypt();
    if (!encrypt) {
      return {
        error: 'Encryption function not available.'
      };
    }

    let { response } = await changePassword({
      body: {
        password: encrypt.encrypt(form.password || '') || '',
        password_confirm: encrypt.encrypt(form.password_confirm || '') || ''
      }
    });

    if (response?.status !== 200) {
      if (response?.status === 401) {
        return { error: 'Passwords are not equal', field: 'password_confirm' };
      } else {
        return { error: 'Error while updating password' };
      }
    } else {
      toast.success('Update successful', {
        description: 'Password was changed successfully'
      });
    }
  };
</script>

<div class="flex items-center">
  <h5>Password:</h5>
  <FormDialog
    title="Change Password"
    description="Enter your new password below"
    confirm="Change Password"
    trigger={{
      text: 'Change Password',
      variant: 'secondary',
      class: 'ml-auto',
      loadIcon: true
    }}
    onopen={startChange}
    onsubmit={changeConfirm}
    schema={passwordChange}
  >
    {#snippet children({ props })}
      <FormInput
        {...props}
        label="New Password"
        key="password"
        placeholder="New Password"
        autocapitalize="none"
        autocomplete="new-password"
        autocorrect="off"
        type="password"
      />
      <FormInput
        {...props}
        label="Confirm New Password"
        key="password_confirm"
        placeholder="Confirm New Password"
        autocapitalize="none"
        autocomplete="new-password"
        autocorrect="off"
        type="password"
      />
    {/snippet}
  </FormDialog>
</div>
