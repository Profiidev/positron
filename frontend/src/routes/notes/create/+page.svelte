<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type { Stage } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import { createNote } from '$lib/client';

  let stages: Stage[] = [
    {
      title: 'Create Note',
      content: Information,
      data: {}
    }
  ];

  const submit = async (rawData: object) => {
    let res = await createNote({ body: rawData as { title: string } });

    if (!res.data) {
      return { error: 'Error creating note.' };
    } else {
      toast.success('Note created successfully.');
      setTimeout(() => {
        goto(`/notes/${res.data.id}`);
      });
    }
  };
</script>

<MultiStepForm {stages} onsubmit={submit} cancelHref="/notes" />
