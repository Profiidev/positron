<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import { startRegistration, startAuthentication } from "@simplewebauthn/browser";

  let text = "No Status";

  const begin = async () => {
    let res = await fetch("http://localhost:8000/auth/passkey/start_registration/1234");
    let optionsJSON = (await res.json()).publicKey;

    let resp;
    try {
      resp = await startRegistration({ optionsJSON });
    } catch (error) {
      if(error.name === "InvalidStateError") {
        text = "Already done";
      } else {
        text = error;
      }
      throw error;
    }

    const ver = await fetch("http://localhost:8000/auth/passkey/finish_registration/1234", {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(resp),
    });
    const done = await ver.text();
    text = "Done";
  };

  const auth = async () => {
    let res = await fetch("http://localhost:8000/auth/passkey/start_authentication/1234");
    let optionsJSON = (await res.json()).publicKey;

    let resp;
    try {
      resp = await startAuthentication({ optionsJSON });
    } catch(error) {
      text = error;
      throw error;
    }

    const ver = await fetch("http://localhost:8000/auth/passkey/finish_authentication/1234", {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(resp),
    });
    const done = await ver.text();
    text = "Success";
  }
</script>

<div class="flex w-full h-full items-center justify-center">
  <a href="/login">Login</a>
  <Button on:click={begin}>Start</Button>
  <Button on:click={auth}>Auth</Button>
  <p>{text}</p>
</div>