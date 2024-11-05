import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { get_token, TokenType } from "$lib/auth/token.svelte";
import { base64ToArrayBuffer } from "$lib/util/convert.svelte";
import type { ProfileInfo, UserInfo } from "./types.svelte";

export const info = async (uuid: string) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/account/general/info/${uuid}`, {
      headers: {
        Authorization: token,
      },
    });

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();
    return info as UserInfo;
  } catch (_) {
    return;
  }
}

export const change_image = async (image: string) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/account/general/change_image`, {
      method: "POST",
      headers: {
        "Content-Type": "application/octet-stream",
        Authorization: token,
      },
      body: base64ToArrayBuffer(image),
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
}

export const update_profile = async (profile: ProfileInfo) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/account/general/update_profile`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify(profile),
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
}
