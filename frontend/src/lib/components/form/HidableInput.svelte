<script lang="ts" generics="S extends FormRecord = FormRecord">
  import type {
    FormRecord,
    SuperForm,
    FormPath
  } from '@profidev/pleiades/components/form/types';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import { Label } from '@profidev/pleiades/components/ui/label';
  import type { Snippet } from 'svelte';

  interface Props {
    formData: SuperForm<S>;
    key: FormPath<S>;
    id: string;
    value: any;
    children: Snippet;
  }

  const { formData: form, key, id, value, children }: Props = $props();
  let { form: formData } = $derived(form);
</script>

{#if $formData[key]}
  <Label for={id} class="flex">
    {@render children()}
  </Label>
  <Input {id} {value} readonly />
{/if}
