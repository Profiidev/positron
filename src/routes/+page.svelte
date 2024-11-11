<script lang="ts">
  import { goto } from "$app/navigation";
  import { get_token, TokenType } from "$lib/backend/auth/token.svelte";
  import { onMount } from "svelte";

  onMount(() => {
    if (!get_token(TokenType.Auth)) {
      goto("/login", {
        replaceState: true,
      });
    }
  });
</script>
