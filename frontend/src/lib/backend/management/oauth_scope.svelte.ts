import { ResponseType, get, post } from 'positron-components/backend';
import type {
  OAuthPolicyInfo,
  OAuthScope,
  OAuthScopeCreate
} from './types.svelte';

export const list_scopes = async () => {
  let ret = await get<OAuthScope[]>('/backend/management/oauth_scope/list', {
    res_type: ResponseType.Json
  });

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const delete_scope = async (uuid: string) => {
  return await post('/backend/management/oauth_scope/delete', {
    body: {
      uuid
    }
  });
};

export const create_scope = async (scope: OAuthScopeCreate) => {
  return await post('/backend/management/oauth_scope/create', {
    body: scope
  });
};

export const edit_scope = async (scope: OAuthScope) => {
  return await post('/backend/management/oauth_scope/edit', {
    body: scope
  });
};

export const list_policy_info = async () => {
  let ret = await get<OAuthPolicyInfo[]>(
    '/backend/management/oauth_scope/policy_list',
    {
      res_type: ResponseType.Json
    }
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};
