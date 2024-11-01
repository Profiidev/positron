<script lang="ts">
  import { authenticate, register } from "$lib/auth/passkey";
  import { login } from "$lib/auth/password";
  import Button from "$lib/components/ui/button/button.svelte";

  let text = "No Status";

  const begin = async () => {
    if (await register("test@profidev.io")) {
      text = "Done";
    } else {
      text = "Error";
    }
  };

  const auth = async () => {
    if (await authenticate()) {
      text = "Success";
    } else {
      text = "Error";
    }
  }

  const login_ = async () => {
    await login("test@profidev.io", "test1234");
  }
</script>

<div class="flex w-full h-full items-center justify-center">
  <a href="/login">Login</a>
  <Button on:click={begin}>Start</Button>
  <Button on:click={auth}>Auth</Button>
  <Button on:click={login_}>Login</Button>
  <p>{text}</p>
</div>