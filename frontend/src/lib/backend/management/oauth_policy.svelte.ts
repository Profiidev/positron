import { ResponseType, get, post } from 'positron-components/backend';
import type { OAuthPolicy, OAuthPolicyCreate } from './types.svelte';

export const list_policies = async () => {
  let ret = await get<OAuthPolicy[]>('/backend/management/oauth_policy/list', {
    res_type: ResponseType.Json
  });

  if (Array.isArray(ret)) {
    return ret;
  }
};

export const delete_policy = async (uuid: string) => {
  return await post('/backend/management/oauth_policy/delete', {
    body: {
      uuid
    }
  });
};

export const create_policy = async (policy: OAuthPolicyCreate) => {
  return await post('/backend/management/oauth_policy/create', {
    body: policy
  });
};

export const edit_policy = async (policy: OAuthPolicy) => {
  return await post('/backend/management/oauth_policy/edit', { body: policy });
};
