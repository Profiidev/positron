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
  let res = await post<{ location: string }>(
    `/oauth/authorize_confirm?code=${params.code}&allow=${allow}`,
    ResponseType.Json,
    ContentType.UrlFrom,
    undefined,
  );

  if (typeof res === "object") {
    if (res.location !== "") {
      window.location.href = res.location;
    }
    return;
  } else {
    return res;
  }
};
