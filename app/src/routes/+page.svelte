<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Label } from '@profidev/pleiades/components/ui/label';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { listen } from '@tauri-apps/api/event';

  let name = $state('');
  let greetMsg = $state('');
  let urls: string[] = $state([]);

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke('greet', { name });
  }

  listen('test', (e) => {
    urls = e.payload as string[];
  });
</script>

<div class="grid h-full place-items-center">
  <div>
    <Label>{greetMsg}</Label>
    <Button onclick={greet}>Greet</Button>
  </div>
  {#each urls as url}
    <p>{url}</p>
  {/each}
</div>
