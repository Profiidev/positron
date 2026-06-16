<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type { Stage } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import { createNote } from '$lib/client';

  const { data } = $props();

  onMount(() => {
    Promise.all([data.notes, data.notesConfig]).then(([notes, config]) => {
      const maxPerUser = config?.max_per_user;
      const ownedCount = notes.filter((n) => n.is_owner).length;

      if (maxPerUser !== undefined && ownedCount >= maxPerUser) {
        goto('/notes');
      }
    });
  });

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
      if (res.response?.status === 409) {
        return { error: 'You have reached the maximum number of notes.' };
      }
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
