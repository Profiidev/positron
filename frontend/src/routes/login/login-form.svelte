<script lang="ts">
  import {
    LoginOtherOptions,
    Totp_6,
    BaseForm,
    FormInput,
    type FormSchema
  } from 'positron-components/components/form';
  import { cn } from 'positron-components/utils';
  import { goto } from '$app/navigation';
  import type { OAuthParams } from '$lib/backend/auth/types.svelte';
  import { totp_confirm } from '$lib/backend/auth/totp.svelte';
  import { RequestError } from 'positron-components/backend';
  import { password_login } from '$lib/backend/auth/password.svelte';
  import {
    passkey_authenticate,
    passkey_authenticate_by_email
  } from '$lib/backend/auth/passkey.svelte';
  import { connect_updater } from '$lib/backend/ws/updater.svelte';
  import type { SuperValidated } from 'sveltekit-superforms';
  import type { SvelteComponent } from 'svelte';

  interface Props {
    class?: string | undefined | null;
    oauth_params: OAuthParams | undefined;
    loginForm: FormSchema<any>;
  }

  let {
    class: className = undefined,
    oauth_params,
    loginForm
  }: Props = $props();

  let passkeySecondTry = $state(false);
  let enterEmail = $state(true);
  let isLoading = $state(false);
  let passkeyError = $state('');
  let formComp: SvelteComponent | undefined = $state();

  const onSubmit = async (form: SuperValidated<any>) => {
    if (passkeySecondTry) {
      passkeyError = '';

      let ret = await passkey_authenticate_by_email(form.data.passkey_email);

      if (ret) {
        if (ret === RequestError.Unauthorized) {
          passkeyError = 'There was an Error with your passkey';
        } else {
          passkeyError = 'There was an Error while signing in';
        }
      } else {
        login_success();
      }
    }

    if (!enterEmail) {
      passkeyError = '';

      let ret = await totp_confirm(form.data.totp);

      if (ret) {
        if (ret === RequestError.Unauthorized) {
          return { field: 'totp', error: 'Wrong TOTP Code' };
        } else {
          return { error: 'There was and Error while checking TOTP Code' };
        }
      } else {
        login_success();
        return;
      }
    }

    passkeyError = '';

    let ret = await password_login(form.data.email, form.data.password);

    if (typeof ret === 'boolean') {
      if (ret) {
        enterEmail = false;
        formComp?.setValue({
          code_input: true
        });
        return { error: '' };
      } else {
        login_success();
      }
    } else {
      if (ret === RequestError.Unauthorized) {
        return { field: 'password', error: 'Wrong Email or Password' };
      } else {
        return { error: 'There was an Error while signing in' };
      }
    }
  };

  const passkeyClick = async () => {
    passkeyError = '';
    isLoading = true;

    let ret = await passkey_authenticate();

    isLoading = false;

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        passkeyError = 'There was an Error with your passkey';
      } else {
        formComp?.setValue({
          passkey_email_input: true
        });
      }
    } else {
      login_success();
    }
  };

  const login_success = async () => {
    connect_updater();
    if (oauth_params) {
      goto(`/oauth?code=${oauth_params.code}&name=${oauth_params.name}`);
    } else {
      setTimeout(() => {
        goto('/');
      });
    }
  };
</script>

<div class={cn('grid gap-6', className)}>
  <BaseForm
    bind:this={formComp}
    onsubmit={onSubmit}
    confirm={enterEmail ? 'Sign In' : 'Confirm'}
    bind:isLoading
    form={loginForm}
    class="gap-0"
  >
    {#snippet children({ props })}
      {#if passkeySecondTry}
        <FormInput
          key="passkey_email"
          label="Email"
          placeholder="name@example.com"
          autocapitalize="none"
          autocomplete="email"
          autocorrect="off"
          {...props}
        />
      {:else if enterEmail}
        <FormInput
          key="email"
          label="Email"
          placeholder="name@example.com"
          autocapitalize="none"
          autocomplete="email"
          autocorrect="off"
          {...props}
        />
        <FormInput
          key="password"
          label="Password"
          placeholder="Password"
          autocapitalize="none"
          autocomplete="current-password"
          autocorrect="off"
          type="password"
          {...props}
        />
      {:else}
        <Totp_6
          label="TOTP"
          key="totp"
          class="flex w-full justify-between sm:w-[350px]"
          {...props}
        />
      {/if}
    {/snippet}
    {#snippet footer({ children })}
      {@render children({ className: 'mt-2' })}
    {/snippet}
  </BaseForm>
  <LoginOtherOptions {isLoading} {passkeyError} {passkeyClick} />
</div>
