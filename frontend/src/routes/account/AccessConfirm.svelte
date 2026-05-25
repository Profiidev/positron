<script lang="ts">
  import LoginOtherOptions from '@profidev/pleiades/components/form/login-other-options.svelte';
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { interval } from '@profidev/pleiades/util/interval.svelte';
  import * as Dialog from '@profidev/pleiades/components/ui/dialog';
  import { browser } from '$app/environment';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { confirmSchema } from './schema.svelte';
  import { getEncrypt } from '$lib/backend/auth.svelte';
  import {
    finishSpecialAccess,
    passwordSpecialAccess,
    startSpecialAccess
  } from '$lib/client';
  import {
    type AuthenticationResponseJSON,
    type PublicKeyCredentialRequestOptionsJSON
  } from '@simplewebauthn/browser';
  import { toast } from '@profidev/pleiades/components/util/general';
  import FormInputPassword from '@profidev/pleiades/components/form/form-input-password.svelte';

  interface Props {
    specialAccessValid: boolean;
  }

  let { specialAccessValid = $bindable(false) }: Props = $props();

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
  let passkeyError = $state(false);
  let isLoading = $state(false);

  export const requestAccess = async () => {
    return new Promise<boolean>((resolve) => {
      cb = resolve;
      open = true;
    });
  };

  const confirm = async (form: FormValue<typeof confirmSchema>) => {
    passkeyError = false;
    let encrypt = getEncrypt();
    if (!encrypt) {
      return {
        error: 'Encryption function not available. Please try again later.'
      };
    }

    let ret = await passwordSpecialAccess({
      body: {
        password: encrypt.encrypt(form.password) || ''
      }
    });

    if (ret.response?.status !== 200) {
      if (ret.response?.status === 401) {
        return { error: 'Wrong Password', field: 'password' } as const;
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
    passkeyError = false;

    let { data, response } = await startSpecialAccess();
    if (response?.status !== 200) {
      isLoading = false;
      passkeyError = true;
      toast.error('There was an error while starting passkey authentication');
      return;
    }

    let optionsJSON = (
      data as { publicKey: PublicKeyCredentialRequestOptionsJSON }
    ).publicKey;

    let authRes: AuthenticationResponseJSON;
    try {
      const startAuthentication = (await import('@simplewebauthn/browser'))
        .startAuthentication;
      authRes = await startAuthentication({ optionsJSON });
    } catch {
      isLoading = false;
      passkeyError = true;
      toast.error('Authentication failed or was cancelled');
      return;
    }

    let { response: authResponse } = await finishSpecialAccess({
      body: authRes
    });

    isLoading = false;

    if (authResponse?.status !== 200) {
      if (authResponse?.status === 401) {
        passkeyError = true;
        toast.error('There was an error with your passkey');
      } else {
        passkeyError = true;
        toast.error('There was an error while confirming access');
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
  <Dialog.Content class="grid w-87 gap-6">
    <Dialog.Header>
      <Dialog.Title>Confirm Access</Dialog.Title>
      <Dialog.Description>Confirm access to your account</Dialog.Description>
    </Dialog.Header>
    <BaseForm
      onsubmit={confirm}
      bind:isLoading
      schema={confirmSchema}
      submitText="Confirm Access"
    >
      {#snippet children({ props })}
        <FormInputPassword
          {...props}
          label="Password"
          key="password"
          placeholder="Password"
        />
      {/snippet}
    </BaseForm>
    <LoginOtherOptions {isLoading} {passkeyError} {passkeyClick} />
  </Dialog.Content>
</Dialog.Root>
