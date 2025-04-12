<script lang="ts">
  import { Input, Label } from 'positron-components/components/ui';
    import type { Snippet } from 'svelte';
    import type { SuperForm } from 'sveltekit-superforms';

  interface Props {
    formData: SuperForm<any>;
    key: string;
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
  <Input id={id} value={value} readonly />
{/if}
