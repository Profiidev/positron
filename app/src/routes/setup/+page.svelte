<script lang="ts">
  import MultistepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import type {
    FormValue,
    Stage
  } from '@profidev/pleiades/components/form/types';
  import InstanceUrl from './InstanceUrl.svelte';
  import Check from '@lucide/svelte/icons/check';
  import type { instanceUrl } from './schema.svelte';
  import { setup } from '$lib/commands/setup.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';

  const stages: Stage[] = [
    {
      title: 'Connect to Positron',
      content: InstanceUrl,
      data: {}
    }
  ];

  const onsubmit = async (rawData: object) => {
    const url = (rawData as FormValue<typeof instanceUrl>).url;

    try {
      const reqUrl = new URL(url);
      reqUrl.pathname = '/api/health';
      const response = await fetch(reqUrl);
      if (response.status !== 200 || !response.headers.get('X-Api-Version')) {
        return {
          error: 'Failed to connect to Positron',
          field: 'url'
        };
      }
    } catch (e) {
      return {
        error: 'Failed to connect to Positron',
        field: 'url'
      };
    }

    if (!(await setup(url))) {
      return {
        error: 'Failed to setup Positron'
      };
    }

    toast.success('Positron setup successful.');
    setTimeout(() => {
      goto('/auth');
    });
  };
</script>

<MultistepForm
  cancelHref="/"
  {stages}
  {onsubmit}
  submitLabel="Confirm"
  submitIcon={Check}
/>
