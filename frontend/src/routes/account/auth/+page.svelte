<script lang="ts">
  import Password from './Password.svelte';
  import AccessConfirm from '../AccessConfirm.svelte';
  import type { SvelteComponent } from 'svelte';
  import PasskeyList from './PasskeyList.svelte';
  import Totp2fa from './Totp2fa.svelte';

  const { data } = $props();

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false)
  );
</script>

<h4 class="mb-2">Authentication</h4>
<Password {requestAccess} valid={specialAccessValid} />
<h5 class="my-2">Passkeys:</h5>
<PasskeyList
  {requestAccess}
  valid={specialAccessValid}
  passkeys={data.passkeys}
/>
<h5 class="my-2">Other 2FA Methods::</h5>
<Totp2fa {requestAccess} valid={specialAccessValid} userInfo={data.user} />
<AccessConfirm bind:specialAccessValid bind:this={accessConfirm} />
