<script lang="ts">
  import {
    LoginOtherOptions,
    BaseForm,
    FormInput,
    type FormSchema
  } from 'positron-components/components/form';
  import { interval } from 'positron-components/util';
  import { RequestError } from 'positron-components/backend';
  import { Dialog } from 'positron-components/components/ui';
  import { password_special_access } from '$lib/backend/auth/password.svelte';
  import { passkey_special_access } from '$lib/backend/auth/passkey.svelte';
  import { browser } from '$app/environment';
  import type { SuperValidated } from 'sveltekit-superforms';

  interface Props {
    specialAccessValid: boolean;
    formData: FormSchema<any>;
  }

  let { specialAccessValid = $bindable(false), formData }: Props = $props();

  let specialAccessWatcher = interval(() => {
    if (!browser) {
      return;
    }

    let match = document.cookie.match(
      new RegExp('(^| )' + 'special_valid' + '=([^;]+)')
    );
    if (match) return Boolean(match[2]);
  }, 1000);
  $effect(() => {
    specialAccessValid =
      specialAccessWatcher.value !== undefined && specialAccessWatcher.value;
  });

  let cb: (value: boolean) => void;
  let open = $state(false);
  let passkeyError = $state('');
  let isLoading = $state(false);

  export const requestAccess = async () => {
    return new Promise<boolean>((resolve) => {
      cb = resolve;
      open = true;
    });
  };

  const confirm = async (form: SuperValidated<any>) => {
    passkeyError = '';

    let ret = await password_special_access(form.data.password);

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return { error: 'Wrong Password' };
      } else {
        return { error: 'There was an error while confirming access' };
      }
    } else {
      cb(true);
      open = false;
    }
  };

  const passkeyClick = async () => {
    isLoading = true;
    passkeyError = '';

    let ret = await passkey_special_access();

    isLoading = false;

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        passkeyError = 'There was an error with your passkey';
      } else {
        passkeyError = 'There was an error while signing in';
      }
    } else {
      cb(true);
      open = false;
    }
  };

  const onOpenChange = (open: boolean) => {
    if (!open) {
      cb(false);
    }
  };
</script>

<Dialog.Root {onOpenChange} bind:open>
  <Dialog.Content class="grid w-[350px] gap-6">
    <Dialog.Header>
      <Dialog.Title>Confirm Access</Dialog.Title>
      <Dialog.Description>Confirm access to your account</Dialog.Description>
    </Dialog.Header>
    <BaseForm
      onsubmit={confirm}
      confirm="Confirm Access"
      bind:isLoading
      form={formData}
    >
      {#snippet children({ props })}
        <FormInput
          {...props}
          label="Password"
          key="password"
          placeholder="Password"
          autocapitalize="none"
          autocomplete="current-password"
          autocorrect="off"
          type="password"
        />
      {/snippet}
      {#snippet footer({ children })}
        {@render children()}
      {/snippet}
    </BaseForm>
    <LoginOtherOptions {isLoading} {passkeyError} {passkeyClick} />
  </Dialog.Content>
</Dialog.Root>
