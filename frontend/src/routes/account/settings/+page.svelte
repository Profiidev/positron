<script lang="ts">
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import { generalSettings } from './schema.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import Save from '@lucide/svelte/icons/save';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { saveAccountSettings } from '$lib/client';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import FormSwitch from '@profidev/pleiades/components/form/form-switch.svelte';

  let { data } = $props();

  let form: BaseForm<typeof generalSettings> | undefined = $state();
  let readonly = $state(true);

  $effect(() => {
    data.settings.then((d) => {
      if (d) {
        form?.setValue(d);
        readonly = false;
      }
    });
  });

  const onsubmit = async (form: FormValue<typeof generalSettings>) => {
    let ret = await saveAccountSettings({ body: form });

    if (ret.error) {
      return {
        error: 'Failed to save account settings'
      };
    } else {
      toast.success('General settings saved successfully');
    }
    // do not trigger form reset
    return { error: '' };
  };
</script>

<h4 class="mb-2">Settings</h4>
<BaseForm schema={generalSettings} {onsubmit} bind:this={form}>
  {#snippet children({ props })}
    <div class="grid grid-cols-1 gap-8 lg:grid-cols-2">
      <div class="flex flex-col gap-2">
        <FormSwitch
          {...props}
          key="o_auth_instant_confirm"
          label="Skip confirmation for OAuth logins"
          disabled={readonly}
        />
      </div>
    </div>
  {/snippet}
  {#snippet footer({
    isLoading,
    isError
  }: {
    isLoading: boolean;
    isError: boolean;
  })}
    <div class="mt-4 grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
      <Button
        class="ml-auto cursor-pointer"
        type="submit"
        disabled={isLoading}
        variant={isError ? 'destructive' : undefined}
      >
        {#if isLoading}
          <Spinner />
        {:else if isError}
          <RotateCcw />
        {:else}
          <Save />
        {/if}
        {isError ? 'Retry' : 'Save Changes'}</Button
      >
    </div>
  {/snippet}
</BaseForm>
