import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { getEncrypt } from "../auth/password.svelte";
import { get_token, TokenType } from "../auth/token.svelte";
import type { Permission, User } from "./types.svelte";

export const list = async () => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/management/user/list`, {
      headers: {
        Authorization: token,
      },
    });

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();
    return info as User[];
  } catch (_) {
    return;
  }
};

export const update_permissions = async (
  user: string,
  permission: Permission,
  add: boolean,
) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/management/user/edit`, {
      method: "POST",
      headers: {
        Authorization: token,
      },
      body: JSON.stringify({
        user,
        add_permission: add ? permission : undefined,
        remove_permission: !add ? permission : undefined,
      }),
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};

export const create = async (name: string, email: string, password: string) => {
  let token = get_token(TokenType.Auth);
  let encrypt = getEncrypt();
  if (!token || !encrypt) {
    return;
  }

  try {
    let encrypted_password = encrypt.encrypt(password);

    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/management/user/create`, {
      method: "POST",
      headers: {
        Authorization: token,
      },
      body: JSON.stringify({
        name,
        email,
        password: encrypted_password,
      }),
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};

export const remove = async (uuid: string) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/management/user/delete`, {
      method: "POST",
      headers: {
        Authorization: token,
      },
      body: JSON.stringify({
        uuid,
      }),
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};