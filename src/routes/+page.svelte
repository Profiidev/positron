<script lang="ts">
  import { authenticate, register } from "$lib/auth/passkey.svelte";
  import { login } from "$lib/auth/password.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  let text = $state("No Status");

  const begin = async () => {
    if (await register()) {
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
    if(await login("test@profidev.io", "test1234")) {
      text = "Login";
    } else {
      text = "error";
    }
  }
</script>

<div class="flex w-full h-full items-center justify-center">
  <a href="/login">Login</a>
  <Button onclick={begin}>Start</Button>
  <Button onclick={auth}>Auth</Button>
  <Button onclick={login_}>Login</Button>
  <p>{text}</p>
</div>