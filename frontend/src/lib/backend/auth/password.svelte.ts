import { PUBLIC_BACKEND_URL } from "$env/static/public";
import type JSEncrypt from "jsencrypt";
import { AuthError, type PasswordInfo } from "./types.svelte";
import { browser } from "$app/environment";

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => {
  return encrypt;
};

export const fetch_key = async (): Promise<AuthError | undefined> => {
  if (encrypt === false) {
    return AuthError.Other;
  }

  try {
    let key_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/key`);

    if (key_res.status !== 200) {
      return AuthError.Other;
    }

    let key_pem = await key_res.text();

    const JSEncrypt = (await import("jsencrypt")).JSEncrypt;

    encrypt = new JSEncrypt({ default_key_size: "4096" });
    encrypt.setPublicKey(key_pem);
  } catch (_) {
    return AuthError.Other;
  }
};

fetch_key();

export const login = async (
  email: string,
  password: string,
): Promise<AuthError | boolean> => {
  if (!encrypt) {
    return AuthError.Other;
  }

  try {
    let encrypted_password = encrypt.encrypt(password);

    let login_res = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/password/authenticate`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          email,
          password: encrypted_password,
        }),
      },
    );

    if (login_res.status === 401) {
      return AuthError.Password;
    }

    if (login_res.status !== 200) {
      fetch_key();
      return AuthError.Other;
    }

    let totp = Boolean(await login_res.text());
    if (totp) {
      return true;
    } else {
      return false;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const special_access = async (
  password: string,
): Promise<AuthError | undefined> => {
  if (!encrypt) {
    return AuthError.Other;
  }

  try {
    let encrypted_password = encrypt.encrypt(password);

    let login_res = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/password/special_access`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          password: encrypted_password,
        }),
      },
    );

    if (login_res.status === 401) {
      return AuthError.Password;
    }

    if (login_res.status !== 200) {
      fetch_key();
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const change = async (
  password: string,
  password_confirm: string,
): Promise<AuthError | undefined> => {
  if (!encrypt) {
    return AuthError.Other;
  }

  try {
    let encrypted_password = encrypt.encrypt(password);
    let encrypted_password_confirm = encrypt.encrypt(password_confirm);

    let login_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/change`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        password: encrypted_password,
        password_confirm: encrypted_password_confirm,
      }),
    });

    if (login_res.status === 409) {
      return AuthError.Password;
    }

    if (login_res.status !== 200) {
      fetch_key();
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const info = async (): Promise<undefined | PasswordInfo> => {
  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/password/info`);

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();

    return info as PasswordInfo;
  } catch (_) {
    return;
  }
};
