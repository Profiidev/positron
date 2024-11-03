import { PUBLIC_BACKEND_URL } from "$env/static/public";
import JSEncrypt from "jsencrypt";
import { get_token, get_token_type, set_token, TokenType } from "./token.svelte";
import { AuthError } from "./error.svelte";

let encrypt = $state(new JSEncrypt({ default_key_size: "4096" }));

export const fetch_key = async (): Promise<AuthError | undefined> => {
  try {
    let key_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/key`);

    if (key_res.status !== 200) {
      return AuthError.Other;
    }

    let key_pem = await key_res.text();

    encrypt.setPublicKey(key_pem);
  } catch (_) {
    return AuthError.Other;
  }
}

export const login = async (email: string, password: string): Promise<AuthError | boolean> => {
  try {
    let encrypted_password = encrypt.encrypt(password);

    let login_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/authenticate`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email,
        password: encrypted_password,
      }),
    });

    if (login_res.status === 401) {
      return AuthError.Password;
    }

    if (login_res.status !== 200) {
      return AuthError.Other;
    }

    let token = await login_res.text();

    let type = get_token_type(token);
    set_token(token, type);

    if (type === TokenType.TotpRequired) {
      return true;
    } else {
      return false;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const special_access = async (password: string): Promise<AuthError | undefined> => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return AuthError.MissingToken;
  }

  try {
    let encrypted_password = encrypt.encrypt(password);

    let login_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/special_access`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify({
        password: encrypted_password,
      }),
    });

    if (login_res.status === 401) {
      return AuthError.Password;
    }

    if (login_res.status !== 200) {
      return AuthError.Other;
    }

    let special_access = await login_res.text();
    set_token(special_access, TokenType.SpecialAccess);
  } catch (_) {
    return AuthError.Other;
  }
};