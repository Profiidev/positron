<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { type StageProps } from '@profidev/pleiades/components/form/types';
  import { information } from './schema.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';

  let { initialValue, onsubmit, footer, isLoading, data }: StageProps =
    $props();

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
      label="Policy Name"
      placeholder="Enter policy name"
    />
    <FormInput
      {...props}
      key="claim"
      label="Claim Name"
      placeholder="Enter claim name"
    />
    <FormInput
      {...props}
      key="default"
      label="Default Value"
      placeholder="Enter default value"
    />
  {/snippet}
</BaseForm>
