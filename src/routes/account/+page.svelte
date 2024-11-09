<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/account/profile", {
      replaceState: true,
    });
  });
</script>
