import { base64ToArrayBuffer } from "$lib/util/convert.svelte";
import { ContentType, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import type { ProfileInfo, UserInfo } from "./types.svelte";

export const profile_info = async (uuid: string) => {
  return await get<ProfileInfo>(
    `/account/general/profile_info/${uuid}`,
    ResponseType.Json,
  );
};

export const user_info = async () => {
  return await get<UserInfo>("/account/general/info", ResponseType.Json);
};

export const profile_change_image = async (image: string) => {
  return await post<undefined>(
    "/account/general/change_image",
    ResponseType.None,
    ContentType.Bytes,
    base64ToArrayBuffer(image),
  );
};

export const profile_update = async (name: string) => {
  return await post<undefined>(
    "/account/general/update_profile",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name,
    }),
  );
};
