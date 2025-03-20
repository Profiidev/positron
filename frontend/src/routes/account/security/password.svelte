<script lang="ts">
  import {
    FormDialog,
    FormInput,
    type FormSchema
  } from 'positron-components/components/form';
  import {
    Separator,
    Skeleton,
    toast
  } from 'positron-components/components/ui';
  import { DateTime } from 'positron-components/util';
  import { RequestError } from 'positron-components/backend';
  import { userData } from '$lib/backend/account/info.svelte';
  import type { UserInfo } from '$lib/backend/account/types.svelte';
  import { password_change } from '$lib/backend/auth/password.svelte';
  import type { SuperValidated } from 'sveltekit-superforms';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    formData: FormSchema<any>;
  }

  let { valid, requestAccess, formData }: Props = $props();

  let userInfo: UserInfo | undefined = $derived(userData.value?.[0]);

  const startChange = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    return true;
  };

  const changeConfirm = async (form: SuperValidated<any>) => {
    if (form.data.password !== form.data.password_confirm) {
      return { error: 'Passwords are not equal', field: 'password_confirm' };
    }

    let ret = await password_change(
      form.data.password,
      form.data.password_confirm
    );

    if (ret) {
      if (ret === RequestError.Unauthorized) {
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
  <div class="mr-2 flex h-6 space-x-2">
    {#if userInfo}
      <p class="text-muted-foreground text-sm">
        Last login {DateTime.fromISO(userInfo.last_login).toLocaleString(
          DateTime.DATE_MED
        )}
      </p>
      <Separator orientation={'vertical'} />
      <p class="text-muted-foreground text-sm">
        Last special access {DateTime.fromISO(
          userInfo.last_special_access
        ).toLocaleString(DateTime.DATE_MED)}
      </p>
    {:else}
      <Skeleton class="h-6 w-40" />
      <Separator orientation={'vertical'} />
      <Skeleton class="h-6 w-56" />
    {/if}
  </div>
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
    form={formData}
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
