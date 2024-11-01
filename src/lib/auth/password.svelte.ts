import { PUBLIC_BACKEND_URL } from "$env/static/public";
import JSEncrypt from "jsencrypt";
import { set_token } from "./token.svelte";

export const login = async (email: string, password: string) => {
  try {
    let key_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/start_authentication`);
    let key_pem = await key_res.text();


    let encrypt = new JSEncrypt({ default_key_size: "4096" });
    encrypt.setPublicKey(key_pem);
    let encrypted_password = encrypt.encrypt(password);

    let login_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/finish_authentication`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email,
        password: encrypted_password,
      }),
    });
    let token = await login_res.text();
    set_token(token);

    return true;
  } catch (_) {
    return false;
  }
};
