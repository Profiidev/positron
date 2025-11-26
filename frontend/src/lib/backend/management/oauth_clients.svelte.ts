import { ResponseType, get, post } from 'positron-components/backend';
import type {
  GroupInfo,
  OAuthClientCreate,
  OAuthClientInfo,
  UserInfo
} from './types.svelte';

const isCreate = (object: any): object is OAuthClientCreate => {
  return typeof object === 'object' && object !== null && 'secret' in object;
};

export const get_frontend_url = async () => {
  let ret = await get<string>(
    '/backend/management/oauth_client/frontend_url',
    ResponseType.Text
  );

  if (typeof ret === 'string') {
    return ret;
  }
};

export const list_clients = async () => {
  let ret = await get<OAuthClientInfo[]>(
    '/backend/management/oauth_client/list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const list_clients_group = async () => {
  let ret = await get<GroupInfo[]>(
    '/backend/management/oauth_client/group_list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const list_clients_user = async () => {
  let ret = await get<UserInfo[]>(
    '/backend/management/oauth_client/user_list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_client = async (client: OAuthClientInfo) => {
  return await post<undefined>(
    '/backend/management/oauth_client/edit',
    ResponseType.None,
    client
  );
};

export const start_create_client = async () => {
  let ret = await post<OAuthClientCreate>(
    '/backend/management/oauth_client/start_create',
    ResponseType.Json,
    undefined
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
  confidential: boolean
) => {
  return await post<undefined>(
    '/backend/management/oauth_client/create',
    ResponseType.None,
    {
      name,
      redirect_uri,
      additional_redirect_uris,
      scope,
      confidential
    }
  );
};

export const delete_client = async (client_id: string) => {
  return await post<undefined>(
    '/backend/management/oauth_client/delete',
    ResponseType.None,
    {
      uuid: client_id
    }
  );
};

export const reset_client_secret = async (client_id: string) => {
  let ret = await post<{ secret: string }>(
    '/backend/management/oauth_client/reset',
    ResponseType.Json,
    {
      client_id
    }
  );

  if (typeof ret === 'object') {
    return ret;
  }
};

export const list_scope_names = async () => {
  let ret = await get<string[]>(
    '/backend/management/oauth_client/list_scopes',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};
