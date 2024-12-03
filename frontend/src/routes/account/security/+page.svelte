<script lang="ts">
  import Separator from "$lib/components/ui/separator/separator.svelte";
  import PasskeyList from "./passkey-list.svelte";
  import Totp_2fa from "./totp-2fa.svelte";
  import AccessConfirm from "../access-confirm.svelte";
  import Password from "./password.svelte";
  import type { SvelteComponent } from "svelte";
  import type { PageServerData } from "./$types";
  import {
    confirmSchema,
    passkeyCreateSchema,
    passkeyDeleteSchema,
    passkeyEditSchema,
    passwordChange,
  } from "./schema.svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let specialAccessValid: boolean = $state(false);
  let accessConfirm: SvelteComponent | undefined = $state();
  let requestAccess: () => Promise<boolean> = $derived(
    accessConfirm?.requestAccess || (() => false),
  );

  const passkeyCreate = {
    form: data.passkeyCreateForm,
    schema: passkeyCreateSchema,
  };

  const passkeyEdit = {
    form: data.passkeyEditForm,
    schema: passkeyEditSchema,
  };

  const passkeyDelete = {
    form: data.passkeyDeleteForm,
    schema: passkeyDeleteSchema,
  };

  const confirm = {
    form: data.confirmForm,
    schema: confirmSchema,
  };

  const passwordChangeForm = {
    form: data.passwordChange,
    schema: passwordChange,
  };
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
      formData={passwordChangeForm}
    />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Passkey</h3>
    <PasskeyList
      valid={specialAccessValid}
      {requestAccess}
      createSchema={passkeyCreate}
      editSchema={passkeyEdit}
      deleteSchema={passkeyDelete}
    />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Other 2FA Methods</h3>
    <div class="border p-2 rounded-xl">
      <Totp_2fa valid={specialAccessValid} {requestAccess} />
    </div>
  </div>
</div>
<AccessConfirm
  bind:specialAccessValid
  bind:this={accessConfirm}
  formData={confirm}
/>
