<script lang="ts">
  import { createEventListener } from '$lib/backend/auth/passkey.svelte';
  import {
    FormDialog,
    FormInput,
    type FormSchema
  } from 'positron-components/components/form';
  import type { SuperValidated } from 'sveltekit-superforms';

  interface Props {
    form: FormSchema<any>;
  }

  let { form }: Props = $props();

  let pinDialog: boolean = $state(false);

  const pinSend = async (form: SuperValidated<any>) => {
    try {
      const sendPin = (await import('tauri-plugin-webauthn-api')).sendPin;
      await sendPin(form.data.pin);
    } catch (e) {
      if (e instanceof Error) {
        return { error: e.message };
      }
    }
  };

  const openPinDialog = () => {
    pinDialog = true;
  };

  let { pinDescription } = createEventListener(openPinDialog);
</script>

<FormDialog
  title="Enter your PIN"
  description={pinDescription()}
  confirm="Confirm"
  trigger={undefined}
  onsubmit={pinSend}
  bind:open={pinDialog}
  {form}
>
  {#snippet children({ props })}
    <FormInput
      label="PIN"
      placeholder="PIN"
      type="password"
      key="pin"
      {...props}
    />
  {/snippet}
</FormDialog>
