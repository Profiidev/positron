<script lang="ts">
  import { Separator } from 'positron-components/components/ui';
  import PasskeyList from './passkey-list.svelte';
  import Totp_2fa from './totp-2fa.svelte';
  import AccessConfirm from '../access-confirm.svelte';
  import Password from './password.svelte';
  import type { SvelteComponent } from 'svelte';
  import type { PageServerData } from './$types';
  import {
    confirmSchema,
    passkeyCreateSchema,
    passkeyDeleteSchema,
    passkeyEditSchema,
    passwordChange,
    totpAdd,
    totpRemove
  } from './schema.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false)
  );
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-xl font-medium">Security</h3>
    <p class="text-muted-foreground text-sm">
      Change your authentication settings here
    </p>
  </div>
  <Separator />
  <div class="space-y-3">
    <h3 class="text-lg">Password</h3>
    <Password
      valid={specialAccessValid}
      {requestAccess}
      form={data.passwordChange}
      schema={passwordChange}
    />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Passkey</h3>
    <PasskeyList
      valid={specialAccessValid}
      {requestAccess}
      createForm={data.passkeyCreateForm}
      createSchema={passkeyCreateSchema}
      editForm={data.passkeyEditForm}
      editSchema={passkeyEditSchema}
      deleteForm={data.passkeyDeleteForm}
      deleteSchema={passkeyDeleteSchema}
    />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Other 2FA Methods</h3>
    <div class="rounded-xl border p-2">
      <Totp_2fa
        valid={specialAccessValid}
        {requestAccess}
        addForm={data.totpAdd}
        addSchema={totpAdd}
        removeForm={data.totpRemove}
        removeSchema={totpRemove}
      />
    </div>
  </div>
</div>
<AccessConfirm
  bind:specialAccessValid
  bind:this={accessConfirm}
  form={data.confirmForm}
  schema={confirmSchema}
/>
