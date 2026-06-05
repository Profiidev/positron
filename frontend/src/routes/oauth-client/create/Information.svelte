<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { type StageProps } from '@profidev/pleiades/components/form/types';
  import { information } from './schema.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import FormSwitch from '@profidev/pleiades/components/form/form-switch.svelte';
  import type { SimpleOAuthScopeInfo } from '$lib/client';
  import FormSelect from '@profidev/pleiades/components/form/form-select.svelte';

  let {
    initialValue,
    onsubmit,
    footer,
    isLoading,
    data
  }: StageProps<{
    scopes?: SimpleOAuthScopeInfo[];
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
      label="Client Name"
      placeholder="Enter client name"
    />
    <FormInput
      {...props}
      key="redirect_uri"
      label="Default Redirect URI"
      placeholder="https://example.com/callback"
    />
    <FormSelect
      {...props}
      key="scope"
      label="Scopes"
      data={data.scopes?.map((scopes) => ({
        label: scopes.name,
        value: scopes.uuid
      })) ?? []}
    />
    <FormSwitch {...props} key="confidential" label="Confidential Client" />
    <FormSwitch {...props} key="require_pkce" label="Require PKCE" />
  {/snippet}
</BaseForm>
