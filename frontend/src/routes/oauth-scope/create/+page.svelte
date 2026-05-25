<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type {
    FormValue,
    Stage
  } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import { createOAuthScope, type SimpleOAuthPolicyInfo } from '$lib/client';
  import type { information } from './schema.svelte';

  const { data } = $props();

  let stages: Stage<{
    policies?: SimpleOAuthPolicyInfo[];
  }>[] = [
    {
      title: 'Create Scope',
      content: Information,
      data: {}
    }
  ];

  let policies: SimpleOAuthPolicyInfo[] | undefined = $state();

  $effect(() => {
    data.policies.then((d) => {
      policies = d;
    });
  });

  const submit = async (raw: object) => {
    let rawData = raw as FormValue<typeof information>;
    let res = await createOAuthScope({ body: rawData });

    if (!res.data) {
      if (res.response?.status === 409) {
        return {
          error: 'A scope with this name already exists.',
          field: 'name'
        };
      } else if (res.response?.status === 406) {
        return {
          error: 'A scope with this value already exists.',
          field: 'scope'
        };
      } else {
        return { error: 'Error creating scope.' };
      }
    } else {
      toast.success('Scope created successfully.');
      setTimeout(() => {
        goto(`/oauth-scope/${res.data.uuid}`);
      });
    }
  };
</script>

<MultiStepForm
  {stages}
  data={{
    policies
  }}
  onsubmit={submit}
  cancelHref="/oauth-scope"
/>
