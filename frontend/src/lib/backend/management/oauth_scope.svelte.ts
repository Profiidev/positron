import { ResponseType, get, post } from 'positron-components/backend';
import type {
  OAuthPolicyInfo,
  OAuthScope,
  OAuthScopeCreate
} from './types.svelte';

export const list_scopes = async () => {
  let ret = await get<OAuthScope[]>(
    '/backend/management/oauth_scope/list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const delete_scope = async (uuid: string) => {
  return await post<undefined>(
    '/backend/management/oauth_scope/delete',
    ResponseType.None,
    {
      uuid
    }
  );
};

export const create_scope = async (scope: OAuthScopeCreate) => {
  return await post<undefined>(
    '/backend/management/oauth_scope/create',
    ResponseType.None,
    scope
  );
};

export const edit_scope = async (scope: OAuthScope) => {
  return await post<undefined>(
    '/backend/management/oauth_scope/edit',
    ResponseType.None,
    scope
  );
};

export const list_policy_info = async () => {
  let ret = await get<OAuthPolicyInfo[]>(
    '/backend/management/oauth_scope/policy_list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};
