import { ContentType, ResponseType } from "../types.svelte";
import { post } from "../util.svelte";
import type { OAuthParams } from "./types.svelte";

export const logout = async () => {
  return await post<undefined>(
    "/auth/logout",
    ResponseType.None,
    ContentType.Json,
    undefined,
  );
};

export const oauth_auth = async (params: OAuthParams, allow: boolean) => {
  let res = await post<string>(
    `/oauth/authorize?code=${params.code}&allow=${allow}`,
    ResponseType.Text,
    ContentType.UrlFrom,
    undefined,
  );

  if (typeof res === "string") {
    if (res !== "") {
      window.location.href = res;
    }
    return;
  } else {
    return res;
  }
};
