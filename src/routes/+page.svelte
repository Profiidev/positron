<script lang="ts">
  import { AuthError } from "$lib/auth/error.svelte";
  import { authenticate, register, special_access } from "$lib/auth/passkey.svelte";
  import { fetch_key, login, special_access as pw_sa } from "$lib/auth/password.svelte";
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { confirm, confirm_setup, get_setup_code } from "$lib/auth/totp.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  let text = $state("No Status" as string | AuthError);
  let totp = $state("");
  let qr = $state("");
  let code = $state("");

  const key = async () => {
    let res = await fetch_key();
    if(res) {
      text = res;
    } else {
      text = "Fetched Key";
    }
  }

  const passkey_reg = async () => {
    let res = await register();
    if(res) {
      text = res;
    } else {
      text = "Passkey Register Done";
    }
  };

  const passkey_auth = async () => {
    let res = await authenticate();
    if(res) {
      text = res;
    } else {
      text = "Passkey Auth Done";
    }
  };

  const passkey_special = async () => {
    let res = await special_access();
    if(res) {
      text = res;
    } else {
      text = "Passkey Special Done";
    }
  };

  const password_auth = async () => {
    let res = await login("test@profidev.io", "test1234");
    if (typeof res === "boolean") {
      text = `Totp required ${res}`;
    } else {
      text = res;
    }
  };

  const password_special = async () => {
    let res = await pw_sa("test1234");
    if(res) {
      text = res;
    } else {
      text = "Password Special Done";
    }
  };

  const totp_confirm = async () => {
    let res = await confirm(totp);
    if(res) {
      text = res;
    } else {
      text = "Totp Confirm Done";
    }
  };

  const totp_setup = async () => {
    let res = await get_setup_code();
    if(res.qr) {
      qr = res.qr;
      code = res.code;
    } else {
      text = res;
    }
  };

  const totp_setup_done = async () => {
    let res = await confirm_setup(totp);
    if(res) {
      text = res;
    } else {
      text = "Totp Setup Done";
    }
  };

  const reset = () => {
    text = "No Status";
  }
</script>

<div class="flex w-full h-full items-center justify-center flex-col">
  <a href="/login">Login</a>
  <Button onclick={key}>Get Key</Button>
  <Button onclick={passkey_reg}>Passkey Register</Button>
  <Button onclick={passkey_auth}>Passkey Auth</Button>
  <Button onclick={passkey_special}>Passkey Special Access</Button>
  <Button onclick={password_auth}>Password Auth</Button>
  <Button onclick={password_special}>Password Special Access</Button>
  <Button onclick={totp_confirm}>Totp Confirm</Button>
  <Button onclick={totp_setup}>Totp Setup</Button>
  <Button onclick={totp_setup_done}>Totp Setup Done</Button>
  <Button onclick={reset}>Reset</Button>
  <p>Totp</p>
  <input type="text" bind:value={totp} placeholder="TOTP">
  <p>Auth Token</p>
  <p>{get_token(TokenType.Auth)}</p>
  <p>Special Token</p>
  <p>{get_token(TokenType.SpecialAccess)}</p>
  <p>Totp Token</p>
  <p>{get_token(TokenType.TotpRequired)}</p>
  <p>Totp Code</p>
  <p>{code}</p>
  <p>Totp Image</p>
  <img src={`data:image/png;base64, ${qr}`} alt="QR">
  <p>{text}</p>
</div>