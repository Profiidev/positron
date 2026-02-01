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
  let ret = await get<string>('/backend/management/oauth_client/frontend_url', {
    res_type: ResponseType.Text
  });

  if (typeof ret === 'string') {
    return ret;
  }
};

export const list_clients = async () => {
  let ret = await get<OAuthClientInfo[]>(
    '/backend/management/oauth_client/list',
    {
      res_type: ResponseType.Json
    }
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const list_clients_group = async () => {
  let ret = await get<GroupInfo[]>(
    '/backend/management/oauth_client/group_list',
    {
      res_type: ResponseType.Json
    }
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const list_clients_user = async () => {
  let ret = await get<UserInfo[]>(
    '/backend/management/oauth_client/user_list',
    {
      res_type: ResponseType.Json
    }
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_client = async (client: OAuthClientInfo) => {
  return await post('/backend/management/oauth_client/edit', {
    body: client
  });
};

export const start_create_client = async () => {
  let ret = await post<OAuthClientCreate>(
    '/backend/management/oauth_client/start_create',
    {
      res_type: ResponseType.Json
    }
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
  return await post('/backend/management/oauth_client/create', {
    body: {
      name,
      redirect_uri,
      additional_redirect_uris,
      scope,
      confidential
    }
  });
};

export const delete_client = async (client_id: string) => {
  return await post('/backend/management/oauth_client/delete', {
    body: {
      uuid: client_id
    }
  });
};

export const reset_client_secret = async (client_id: string) => {
  let ret = await post<{ secret: string }>(
    '/backend/management/oauth_client/reset',
    {
      res_type: ResponseType.Json,
      body: { client_id }
    }
  );

  if (typeof ret === 'object') {
    return ret;
  }
};

export const list_scope_names = async () => {
  let ret = await get<string[]>(
    '/backend/management/oauth_client/list_scopes',
    {
      res_type: ResponseType.Json
    }
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};
