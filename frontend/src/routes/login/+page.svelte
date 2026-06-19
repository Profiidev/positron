<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import * as Card from '@profidev/pleiades/components/ui/card';
  import { FieldSeparator } from '@profidev/pleiades/components/ui/field';
  import { login, totpSchema } from './schema.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { goto, invalidate } from '$app/navigation';
  import { connectWebsocket } from '$lib/backend/updater.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';
  import FormInputPassword from '@profidev/pleiades/components/form/form-input-password.svelte';
  import {
    appLoginWebsocket,
    cancelAppLogin,
    generateCodeChallenge,
    generateCodeVerifier,
    getEncrypt,
    openAppLoginDeepLink
  } from '$lib/backend/auth.svelte';
  import {
    finishAuthentication,
    passwordAuthenticate,
    retrieveAppToken,
    startAuthentication,
    totpConfirm
  } from '$lib/client';
  import {
    type AuthenticationResponseJSON,
    type PublicKeyCredentialRequestOptionsJSON
  } from '@simplewebauthn/browser';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import Totp6 from '@profidev/pleiades/components/form/totp-6.svelte';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import Smartphone from '@lucide/svelte/icons/smartphone';

  let { data } = $props();

  let passkeyError = $state(false);
  let isLoading = $state(false);
  let totp = $state(false);
  let mailEnabled = $state(false);
  let appError = $state(false);
  let qrCode = $state<string | undefined>(undefined);
  let deviceCode = $state<string | undefined>(undefined);

  $effect(() => {
    data.config?.then((config) => {
      mailEnabled = config?.mail_enabled || false;
    });
  });

  $effect(() => {
    const url = new URL(window.location.href);
    let updated = false;
    if (data.error) {
      let error = '';
      switch (data.error) {
        case 'missing_code':
          error = 'SSO login failed: Missing authorization code.';
          break;
        case 'oidc_not_configured':
          error = 'SSO login failed: OIDC is not configured.';
          break;
        case 'not_found':
          error = 'User not found.';
          break;
        default:
          error = `SSO login failed: ${data.error}`;
      }

      toast.error(error);

      url.searchParams.delete('error');
      updated = true;
    }
    if (updated) {
      window.history.replaceState({}, '', url);
    }
  });

  const loginSuccess = (user: string) => {
    setTimeout(async () => {
      connectWebsocket(user);
      await invalidate('/api/user/info');
      if (data.oauthOptions.code && data.oauthOptions.name) {
        await goto(
          `/oauth?code=${data.oauthOptions.code}&name=${data.oauthOptions.name}`
        );
      } else if (data.auth.authType) {
        const challenge = data.auth.challenge
          ? `?challenge=${data.auth.challenge}`
          : '';
        await goto(`/auth/${data.auth.authType}${challenge}`);
      } else {
        await goto(data.redirectTo);
      }
    });
  };

  const onsubmit = async (formData: FormValue<typeof login>) => {
    let encrypt = getEncrypt();
    if (!encrypt) {
      return {
        error: 'Encryption function not available. Please try again later.'
      };
    }

    let ret = await passwordAuthenticate({
      body: {
        email: formData.email,
        password: encrypt.encrypt(formData.password) || ''
      },
      parseAs: 'json'
    });

    if (!ret.data && ret.response?.status === 401) {
      return {
        error: 'Invalid email or password.',
        field: 'password'
      } as const;
    } else if (!ret.data && ret.response?.status === 429) {
      return { error: 'Rate limit exceeded. Please try again later.' };
    } else if (!ret.data) {
      return { error: 'Login failed. Please try again.' };
    } else {
      let user = (ret.data as { user?: string } | undefined)?.user;
      if (user) {
        loginSuccess(user);
      } else {
        totp = true;
      }
    }
  };

  const confirmTotp = async (formData: FormValue<typeof totpSchema>) => {
    let ret = await totpConfirm({
      body: {
        code: formData.code
      },
      parseAs: 'json'
    });

    if (!ret.data && ret.response?.status === 401) {
      return { error: 'Invalid code.' };
    } else if (!ret.data) {
      return { error: 'Failed to confirm code. Please try again.' };
    } else {
      let user = (ret.data as { user?: string } | undefined)?.user;
      loginSuccess(user || '');
    }
  };

  const passkeyLogin = async () => {
    passkeyError = false;
    isLoading = true;

    let { data, response } = await startAuthentication();
    if (response?.status !== 200) {
      passkeyError = true;
      toast.error('There was an error while starting passkey authentication.');
      isLoading = false;
      return;
    }

    let reqData = data as {
      res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
      id: string;
    };
    let optionsJSON = reqData.res.publicKey;

    let passkeyResponse: AuthenticationResponseJSON;
    try {
      const webauthnStart = (await import('@simplewebauthn/browser'))
        .startAuthentication;
      passkeyResponse = await webauthnStart({ optionsJSON });
    } catch {
      passkeyError = true;
      toast.error('There was an error with your passkey. Please try again.');
      isLoading = false;
      return;
    }

    let { response: regResponse, data: authData } = await finishAuthentication({
      body: passkeyResponse,
      path: {
        auth_id: reqData.id
      }
    });

    if (regResponse?.status !== 200) {
      if (regResponse?.status === 401) {
        passkeyError = true;
        toast.error('There was an error with your passkey');
      } else {
        passkeyError = true;
        toast.error('There was an error while signing in.');
      }
    } else {
      loginSuccess((authData as { user: string }).user);
    }
    isLoading = false;
  };

  const appLogin = async () => {
    appError = false;
    isLoading = true;

    const verifier = generateCodeVerifier();
    const challenge = await generateCodeChallenge(verifier);
    let initialCode = false;

    appLoginWebsocket(
      challenge,
      async (code) => {
        if (!initialCode) {
          initialCode = true;
          deviceCode = code;
          const QRCode = await import('qrcode');
          qrCode = await QRCode.toDataURL(`positron://login?code=${code}`, {
            margin: 1,
            scale: 10
          });
        } else {
          const { data, response } = await retrieveAppToken({
            body: { auth_code: code, verifier }
          });

          if (response?.status !== 200 || !data) {
            appError = true;
            toast.error('There was an error while signing in.');
          } else {
            appError = false;
            loginSuccess((data as { user: string }).user);
          }
          isLoading = false;
        }
      },
      () => {
        appError = true;
        isLoading = false;
      }
    );
  };
</script>

<div class="flex h-screen w-full items-center justify-center px-4">
  <Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header>
      <Card.Title class="text-2xl">Login</Card.Title>
      <Card.Description
        >{qrCode
          ? 'Scan the QR code with your Positron app, or open the app on this device'
          : totp
            ? 'Enter the 6-digit code from your authenticator app to continue'
            : 'Enter your login details below to login'}</Card.Description
      >
    </Card.Header>
    <Card.Content>
      {#if !qrCode}
        {#if totp}
          <BaseForm
            schema={totpSchema}
            bind:isLoading
            onsubmit={confirmTotp}
            submitText="Continue"
          >
            {#snippet children({ props })}
              <Totp6 {...props} key="code" label="" class="justify-evenly" />
            {/snippet}
          </BaseForm>
        {:else}
          <BaseForm schema={login} {onsubmit} bind:isLoading submitText="Login">
            {#snippet children({ props })}
              <FormInput
                {...props}
                label="Email"
                type="email"
                placeholder="mail@example.com"
                key="email"
              />
              <FormInputPassword
                {...props}
                label="Password"
                placeholder="Your password"
                key="password"
              >
                {#if mailEnabled}
                  <a
                    href="/password/forgot"
                    class="ms-auto inline-block text-sm underline"
                    tabindex="-1"
                  >
                    Forgot your password?
                  </a>
                {/if}
              </FormInputPassword>
            {/snippet}
          </BaseForm>
        {/if}
        <FieldSeparator
          class="*:data-[slot=field-separator-content]:bg-card my-4"
          >Or continue with</FieldSeparator
        >
        <Button
          class="w-full cursor-pointer"
          onclick={passkeyLogin}
          disabled={isLoading}
          variant={passkeyError ? 'destructive' : 'outline'}
        >
          {#if isLoading}
            <Spinner />
          {:else if passkeyError}
            <RotateCcw />
          {:else}
            <KeyRound />
          {/if}
          {passkeyError ? 'Retry Passkey' : 'Passkey'}</Button
        >
        <Button
          class="mt-4 w-full cursor-pointer"
          onclick={appLogin}
          disabled={isLoading}
          variant={appError ? 'destructive' : 'outline'}
        >
          {#if isLoading}
            <Spinner />
          {:else if appError}
            <RotateCcw />
          {:else}
            <Smartphone />
          {/if}
          {appError ? 'Retry App Login' : 'App Login'}</Button
        >
      {:else}
        <img src={qrCode} alt="QR Code" class="w-full" />
        <Button
          class="mt-4 w-full cursor-pointer"
          variant="outline"
          disabled={!deviceCode}
          onclick={() => {
            if (deviceCode) {
              openAppLoginDeepLink(deviceCode, window.location.origin);
            }
          }}
        >
          Open in App
        </Button>
        <Button
          class="mt-4 w-full cursor-pointer"
          variant="outline"
          onclick={() => {
            cancelAppLogin();
            qrCode = undefined;
            deviceCode = undefined;
            isLoading = false;
          }}
        >
          Cancel
        </Button>
      {/if}
    </Card.Content>
  </Card.Root>
</div>
