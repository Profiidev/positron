<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import type { Stage } from '@profidev/pleiades/components/form/types';
  import MultiStepForm from '@profidev/pleiades/components/form/multistep-form.svelte';
  import Information from './Information.svelte';
  import Nav from '$lib/components/Nav.svelte';
  import { createNote } from '$lib/commands/notes.svelte';
  import { notesConfigState, notesState } from '$lib/updater/state.svelte';

  const notes = $derived(notesState.value ?? undefined);
  const maxPerUser = $derived(notesConfigState.value?.max_per_user);

  $effect(() => {
    if (
      maxPerUser !== undefined &&
      notes !== undefined &&
      notes.length >= maxPerUser
    ) {
      goto('/');
    }
  });

  let stages: Stage[] = [
    {
      title: 'Create Note',
      content: Information,
      data: {}
    }
  ];

  const submit = async (rawData: object) => {
    const res = await createNote((rawData as { title: string }).title);

    if (!res.ok) {
      if (res.error === 'limit') {
        return { error: 'You have reached the maximum number of notes.' };
      }
      return { error: 'Error creating note.' };
    }

    toast.success('Note created successfully.');
    setTimeout(() => {
      goto(`/notes/${res.id}`);
    });
  };
</script>

<MultiStepForm {stages} onsubmit={submit} cancelHref="/" />
