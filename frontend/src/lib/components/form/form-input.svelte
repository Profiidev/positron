<script lang="ts">
  import * as Form from "$lib/components/ui/form";
  import { type SuperForm } from "sveltekit-superforms";
  import { Input } from "../ui/input";
  import type { HTMLInputTypeAttribute } from "svelte/elements";

  interface Props {
    form: SuperForm<any>;
    key: string;
    label: string;
    type?: HTMLInputTypeAttribute;
    placeholder?: string;
  }

  let { form, key, label, type, placeholder, ...restProps }: Input & Props =
    $props();

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
        {type}
        {placeholder}
      />
    {/snippet}
  </Form.Control>
  <Form.FieldErrors />
</Form.Field>
