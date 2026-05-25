<script lang="ts" generics="T, S extends FormRecord = FormRecord">
  import * as Form from '@profidev/pleiades/components/ui/form';
  import { type FormPath, type SuperForm } from 'sveltekit-superforms';
  import type { FormRecord } from '@profidev/pleiades/components/form/types';
  import { TagsInput } from '@profidev/pleiades/components/ui-extra/tags-input';

  interface Props {
    formData: SuperForm<S>;
    key: FormPath<S>;
    label: string;
    disabled?: boolean;
    suggestions?: string[];
    validate?: (val: string, tags: string[]) => string | undefined;
    onValueChange?: (selected: string[]) => void;
  }

  let {
    formData: form,
    key,
    label,
    disabled,
    onValueChange,
    suggestions,
    validate
  }: Props = $props();

  let formData = $derived(form.form);
</script>

<Form.Field {form} name={key} class="gap-1/2 grid">
  <Form.Control>
    {#snippet children({ props })}
      <Form.Label>{label}</Form.Label>
      {/* @ts-ignore */ null}
      <TagsInput
        placeholder="Add a downstream cache"
        bind:value={$formData[key]}
        {suggestions}
        {validate}
        {disabled}
        {onValueChange}
      />
    {/snippet}
  </Form.Control>
  <Form.FieldErrors />
</Form.Field>
