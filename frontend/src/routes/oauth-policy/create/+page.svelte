<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type {
    FormValue,
    Stage
  } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import { createOAuthPolicy } from '$lib/client';
  import type { information } from './schema.svelte';

  let stages: Stage[] = [
    {
      title: 'Create Policy',
      content: Information,
      data: {}
    }
  ];

  const submit = async (raw: object) => {
    let rawData = raw as FormValue<typeof information>;
    let res = await createOAuthPolicy({ body: rawData });

    if (!res.data) {
      if (res.response?.status === 409) {
        return {
          error: 'A policy with this name already exists.',
          field: 'name'
        };
      } else {
        return { error: 'Error creating policy.' };
      }
    } else {
      toast.success('Policy created successfully.');
      setTimeout(() => {
        goto(`/oauth-policy/${res.data.uuid}`);
      });
    }
  };
</script>

<MultiStepForm {stages} onsubmit={submit} cancelHref="/oauth-policy" />
