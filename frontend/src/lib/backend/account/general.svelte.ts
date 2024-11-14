import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { base64ToArrayBuffer } from "$lib/util/convert.svelte";
import type { ProfileInfo, UserInfo } from "./types.svelte";

export const info = async (uuid: string) => {
  try {
    let info_res = await fetch(
      `${PUBLIC_BACKEND_URL}/account/general/info/${uuid}`,
    );

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();
    return info as UserInfo;
  } catch (_) {
    return;
  }
};

export const get_uuid = async () => {
  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/account/general/uuid`);

    if (info_res.status !== 200) {
      return;
    }

    return await info_res.text();
  } catch (_) {
    return;
  }
};

export const change_image = async (image: string) => {
  try {
    let info_res = await fetch(
      `${PUBLIC_BACKEND_URL}/account/general/change_image`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/octet-stream",
        },
        body: base64ToArrayBuffer(image),
      },
    );

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};

export const update_profile = async (profile: ProfileInfo) => {
  try {
    let info_res = await fetch(
      `${PUBLIC_BACKEND_URL}/account/general/update_profile`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(profile),
      },
    );

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};
