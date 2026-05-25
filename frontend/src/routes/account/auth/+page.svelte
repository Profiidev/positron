<script lang="ts">
  import Password from './Password.svelte';
  import AccessConfirm from '../AccessConfirm.svelte';
  import type { SvelteComponent } from 'svelte';
  import PasskeyList from './PasskeyList.svelte';
  import Totp2fa from './Totp2fa.svelte';
  import type { PasskeyInfo, UserInfo } from '$lib/client';
  import Email from './Email.svelte';

  const { data } = $props();

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let user: UserInfo | undefined = $state();
  let passkeys: PasskeyInfo[] | undefined = $state();
  let mailActive: boolean = $state(false);

  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false)
  );

  $effect(() => {
    data.user.then((userInfo) => {
      user = userInfo;
    });
  });

  $effect(() => {
    data.passkeys.then((passkeyList) => {
      passkeys = passkeyList;
    });
  });

  $effect(() => {
    data.mailActive.then((active) => {
      mailActive = active;
    });
  });
</script>

<div class="mt-4 grid w-full grid-cols-1 gap-8 2xl:grid-cols-2">
  <div>
    <h4 class="mb-2">Authentication</h4>
    <Password {requestAccess} valid={specialAccessValid} />
    <Email
      {requestAccess}
      valid={specialAccessValid}
      email={user?.email ?? ''}
      {mailActive}
    />
    <PasskeyList {requestAccess} valid={specialAccessValid} {passkeys} />
    <h5 class="my-2">Other 2FA Methods::</h5>
    <Totp2fa {requestAccess} valid={specialAccessValid} userInfo={user} />
    <AccessConfirm bind:specialAccessValid bind:this={accessConfirm} />
  </div>
</div>
