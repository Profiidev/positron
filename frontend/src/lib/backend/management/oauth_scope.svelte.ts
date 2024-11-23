import { ContentType, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import type {
  OAuthPolicyInfo,
  OAuthScope,
  OAuthScopeCreate,
} from "./types.svelte";

export const list_scopes = async () => {
  let ret = await get<OAuthScope[]>(
    "/management/oauth_scope/list",
    ResponseType.Json,
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const delete_scope = async (uuid: string) => {
  return await post<undefined>(
    "/management/oauth_scope/delete",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid,
    }),
  );
};

export const create_scope = async (scope: OAuthScopeCreate) => {
  return await post<undefined>(
    "/management/oauth_scope/create",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(scope),
  );
};

export const edit_scope = async (scope: OAuthScope) => {
  return await post<undefined>(
    "/management/oauth_scope/edit",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(scope),
  );
};

export const list_policy_info = async () => {
  let ret = await get<OAuthPolicyInfo[]>(
    "/management/oauth_scope/policy_list",
    ResponseType.Json,
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};
