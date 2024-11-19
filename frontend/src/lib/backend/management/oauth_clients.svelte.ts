import { ContentType, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import type { OAuthClientCreate, OAuthClientInfo } from "./types.svelte";

const isCreate = (object: any): object is OAuthClientCreate => {
  return typeof object === "object" && object !== null && "secret" in object;
};

export const list_clients = async () => {
  let ret = await get<OAuthClientInfo[]>(
    "/management/oauth_client/list",
    ResponseType.Json,
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_client = async (client: OAuthClientInfo) => {
  return await post<undefined>(
    "/management/oauth_client/edit",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(client),
  );
};

export const start_create_client = async () => {
  let ret = await post<OAuthClientCreate>(
    "/management/oauth_client/start_create",
    ResponseType.Json,
    ContentType.Json,
    undefined,
  );

  if (isCreate(ret)) {
    return ret;
  }
};

export const create_client = async (
  name: string,
  redirect_uri: string,
  additional_redirect_uris: string[],
  scope: string,
) => {
  return await post<undefined>(
    "/management/oauth_client/create",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name,
      redirect_uri,
      additional_redirect_uris,
      scope,
    }),
  );
};

export const delete_client = async (client_id: string) => {
  return await post<undefined>(
    "/management/oauth_client/delete",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid: client_id,
    }),
  );
};
