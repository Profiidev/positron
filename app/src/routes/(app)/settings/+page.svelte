<script lang="ts">
  import * as Card from '@profidev/pleiades/components/ui/card';
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import FormSelect from '@profidev/pleiades/components/form/form-select.svelte';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import { settings } from './schema.svelte';
  import {
    HorizontalLayout,
    saveSettings,
    VerticalLayout
  } from '$lib/commands/settings.svelte';
  import { appSettingsState } from '$lib/updater/state.svelte';
  import Save from '@lucide/svelte/icons/save';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import { toast } from '@profidev/pleiades/components/util/general';

  let form = $state<BaseForm<typeof settings>>();
  const appSettings = $derived(appSettingsState.value);

  $effect(() => {
    if (appSettings) {
      form?.setValue({
        horizontal_layout: [appSettings.horizontal_layout],
        vertical_layout: [appSettings.vertical_layout],
        width: appSettings.width,
        height: appSettings.height
      });
    }
  });

  const onsubmit = async (data: FormValue<typeof settings>) => {
    let result = await saveSettings({
      horizontal_layout: data.horizontal_layout[0],
      vertical_layout: data.vertical_layout[0],
      width: data.width,
      height: data.height
    });

    if (result) {
      toast.success('Settings saved successfully.');
      return { error: '' };
    } else {
      return { error: 'Failed to save settings' };
    }
  };
</script>

<div class="grid h-full place-items-center">
  <Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header>
      <Card.Title class="flex">Settings</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col">
      <BaseForm
        schema={settings}
        {onsubmit}
        initialValue={(appSettings && {
          horizontal_layout: [appSettings.horizontal_layout],
          vertical_layout: [appSettings.vertical_layout],
          width: appSettings.width,
          height: appSettings.height
        }) ||
          undefined}
        bind:this={form}
      >
        {#snippet children({ props })}
          <FormSelect
            {...props}
            key="vertical_layout"
            single
            label="Vertical Alignment"
            data={Object.keys(VerticalLayout).map((key) => ({
              label: key,
              value: VerticalLayout[key as keyof typeof VerticalLayout]
            }))}
          />
          <FormSelect
            {...props}
            key="horizontal_layout"
            single
            label="Horizontal Alignment"
            data={Object.keys(HorizontalLayout).map((key) => ({
              label: key,
              value: HorizontalLayout[key as keyof typeof HorizontalLayout]
            }))}
          />
          <FormInput
            {...props}
            key="width"
            type="number"
            min={500}
            label="Width"
          />
          <FormInput
            {...props}
            key="height"
            type="number"
            min={500}
            label="Height"
          />
        {/snippet}
        {#snippet footer({ defaultBtn })}
          {@render defaultBtn({
            className: 'ml-auto',
            content: 'Save',
            icon: Save
          })}
        {/snippet}
      </BaseForm>
    </Card.Content>
  </Card.Root>
</div>
