<script lang="ts">
  import * as Form from "$lib/components/ui/form";
  import { type SuperForm } from "sveltekit-superforms";
  import { Input } from "../ui/input";
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props {
    formData: SuperForm<any>;
    key: string;
    label: string;
    disabled?: boolean;
  }

  let {
    formData: form,
    key,
    label,
    disabled,
    ...restProps
  }: HTMLInputAttributes & Props = $props();

  const { form: formData } = $derived(form);
</script>

<Form.Field {form} name={key}>
  <Form.Control>
    {#snippet children({ props })}
      <Form.Label>{label}</Form.Label>
      <Input {disabled} {...props} {...restProps} bind:value={$formData[key]} />
    {/snippet}
  </Form.Control>
  <Form.FieldErrors />
</Form.Field>
