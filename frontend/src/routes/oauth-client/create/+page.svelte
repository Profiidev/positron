<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type {
    FormValue,
    Stage
  } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import { createOauthClient, type SimpleOAuthScopeInfo } from '$lib/client';
  import type { information } from './schema.svelte';

  const { data } = $props();

  let stages: Stage<{
    scopes?: SimpleOAuthScopeInfo[];
  }>[] = [
    {
      title: 'Create Client',
      content: Information,
      data: {}
    }
  ];

  let scopes: SimpleOAuthScopeInfo[] | undefined = $state();

  $effect(() => {
    data.scopes.then((d) => {
      scopes = d;
    });
  });

  const submit = async (raw: object) => {
    let rawData = raw as FormValue<typeof information>;
    let res = await createOauthClient({ body: rawData });

    if (!res.data) {
      if (res.response?.status === 409) {
        return {
          error: 'A client with this name already exists.',
          field: 'name'
        };
      } else {
        return { error: 'Error creating client.' };
      }
    } else {
      toast.success('Client created successfully.');
      sessionStorage.setItem('newSecret', res.data.client_secret);
      setTimeout(() => {
        goto(`/oauth-client/${res.data.client_id}`);
      });
    }
  };
</script>

<MultiStepForm
  {stages}
  data={{
    scopes
  }}
  onsubmit={submit}
  cancelHref="/oauth-client"
/>
