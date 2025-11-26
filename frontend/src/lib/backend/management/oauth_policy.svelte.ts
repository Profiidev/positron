import { ResponseType, get, post } from 'positron-components/backend';
import type { OAuthPolicy, OAuthPolicyCreate } from './types.svelte';

export const list_policies = async () => {
  let ret = await get<OAuthPolicy[]>(
    '/management/oauth_policy/list',
    ResponseType.Json
  );

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const delete_policy = async (uuid: string) => {
  return await post<undefined>(
    '/management/oauth_policy/delete',
    ResponseType.None,
    {
      uuid
    }
  );
};

export const create_policy = async (policy: OAuthPolicyCreate) => {
  return await post<undefined>(
    '/management/oauth_policy/create',
    ResponseType.None,
    policy
  );
};

export const edit_policy = async (policy: OAuthPolicy) => {
  return await post<undefined>(
    '/management/oauth_policy/edit',
    ResponseType.None,
    policy
  );
};
