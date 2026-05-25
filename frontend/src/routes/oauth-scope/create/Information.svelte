<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { type StageProps } from '@profidev/pleiades/components/form/types';
  import { information } from './schema.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import type { SimpleOAuthPolicyInfo } from '$lib/client';
  import FormSelect from '@profidev/pleiades/components/form/form-select.svelte';

  let {
    initialValue,
    onsubmit,
    footer,
    isLoading,
    data
  }: StageProps<{
    policies?: SimpleOAuthPolicyInfo[];
  }> = $props();

  let form: BaseForm<typeof information> | undefined = $state();

  export const getValue = () => {
    return form?.getValue();
  };
</script>

<BaseForm
  schema={information}
  {onsubmit}
  {footer}
  {initialValue}
  bind:this={form}
  bind:isLoading
>
  {#snippet children({ props })}
    <FormInput
      {...props}
      key="name"
      label="Scope Name"
      placeholder="Enter scope name"
    />
    <FormInput {...props} key="scope" label="Scope" placeholder="Enter scope" />
    <FormSelect
      {...props}
      key="policies"
      label="Policies"
      data={data.policies?.map((policy) => ({
        label: policy.name,
        value: policy.uuid
      })) ?? []}
    />
  {/snippet}
</BaseForm>
