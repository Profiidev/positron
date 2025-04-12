<script lang="ts">
  import { Switch, Form } from 'positron-components/components/ui';
  import type { SuperForm } from 'sveltekit-superforms';

  interface Props {
    formData: SuperForm<any>;
    key: string;
    label: string;
    disabled?: boolean;
  }

  let { formData: form, key, label, disabled, ...restProps }: Props = $props();

  const { form: formData } = $derived(form);
</script>

<Form.Field {form} name={key} class="flex items-center w-full mt-2">
  <Form.Control>
    {#snippet children({ props })}
      <Form.Label>{label}</Form.Label>
      <Switch {...props} {...restProps} bind:checked={$formData[key]} class="ml-auto" />
    {/snippet}
  </Form.Control>
  <Form.FieldErrors />
</Form.Field>
