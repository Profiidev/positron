import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { get_token, TokenType } from "../auth/token.svelte";
import type { Permission } from "../management/types.svelte";

export const list = async () => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(
      `${PUBLIC_BACKEND_URL}/account/permissions/list`,
      {
        headers: {
          Authorization: token,
        },
      },
    );

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();
    return info as Permission[];
  } catch (_) {
    return;
  }
};

export const access_level = async () => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(
      `${PUBLIC_BACKEND_URL}/account/permissions/access_level`,
      {
        headers: {
          Authorization: token,
        },
      },
    );

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.text();
    return Number(info);
  } catch (_) {
    return;
  }
};
