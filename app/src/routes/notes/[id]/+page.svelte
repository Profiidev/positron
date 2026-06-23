<script lang="ts">
  import { page } from '$app/state';
  import Nav from '$lib/components/Nav.svelte';
  import TipTab from '$lib/components/tiptap/TipTab.svelte';
  import { userInfoState } from '$lib/updater/state.svelte';

  const id = $derived(page.params.id ?? '');
  const user = $derived(userInfoState.value);
</script>

<div class="flex h-full flex-col">
  <Nav />
  <div class="flex min-h-0 flex-1 flex-col p-4 pt-0">
    {#key id}
      <TipTab {id} username={user?.name} userId={user?.uuid} />
    {/key}
  </div>
</div>
