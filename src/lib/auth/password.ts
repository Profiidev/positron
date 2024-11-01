import { PUBLIC_BACKEND_URL } from "$env/static/public";
import JSEncrypt from "jsencrypt";

export const login = async (email: string, password: string) => {
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

  return login_res.status === 200;
};
