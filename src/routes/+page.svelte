<script lang="ts">
  import { clear_tokens } from "$lib/auth/token.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  const logout = () => {
    clear_tokens();
  }
</script>

<div class="flex w-full h-full items-center justify-center flex-col">
  <a href="/login">Login</a>
  <a href="/account">Account</a>
  <Button onclick={logout}>Logout</Button>
</div>