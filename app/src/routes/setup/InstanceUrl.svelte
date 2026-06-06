<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { type FormValue } from '@profidev/pleiades/components/form/types';
  import type { ComponentProps, Snippet } from 'svelte';
  import { instanceUrl } from './schema.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';

  interface Props {
    initialValue?: FormValue<typeof instanceUrl>;
    // @ts-ignore
    onsubmit: ComponentProps<BaseForm<any>>['onsubmit'];
    footer: Snippet<[{ isLoading: boolean; isError: boolean }]>;
    isLoading: boolean;
    readonly?: boolean;
  }

  let { initialValue, onsubmit, footer, isLoading, readonly }: Props = $props();

  let form: BaseForm<typeof instanceUrl> | undefined = $state();

  export const getValue = () => {
    return form?.getValue();
  };
</script>

<BaseForm
  schema={instanceUrl}
  {onsubmit}
  {footer}
  {initialValue}
  bind:this={form}
  bind:isLoading
>
  {#snippet children({ props })}
    <FormInput
      {...props}
      key="url"
      label="Instance URL"
      placeholder="https://positron.example.com"
      {readonly}
      required
    />
  {/snippet}
</BaseForm>
