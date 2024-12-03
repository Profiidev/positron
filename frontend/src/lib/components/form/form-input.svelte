<script lang="ts">
  import * as Form from "$lib/components/ui/form";
  import { type SuperForm } from "sveltekit-superforms";
  import { Input } from "../ui/input";
  import type { HTMLInputAttributes, HTMLInputTypeAttribute } from "svelte/elements";

  interface Props {
    formData: SuperForm<any>;
    key: string;
    label: string;
  }

  let {
    formData: form,
    key,
    label,
    ...restProps
  }: HTMLInputAttributes & Props = $props();

  const { form: formData } = $derived(form);
</script>

<Form.Field {form} name={key}>
  <Form.Control>
    {#snippet children({ props })}
      <Form.Label>{label}</Form.Label>
      <Input
        {...props}
        {...restProps}
        bind:value={$formData[key]}
      />
    {/snippet}
  </Form.Control>
  <Form.FieldErrors />
</Form.Field>
