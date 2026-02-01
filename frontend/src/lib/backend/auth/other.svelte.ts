import { ResponseType, get, post } from 'positron-components/backend';
import type { OAuthParams } from './types.svelte';

export const logout = async () => {
  return await post('/auth/logout');
};

export const oauth_auth = async (params: OAuthParams, allow: boolean) => {
  let res = await post<{ location: string }>(
    `/backend/oauth/authorize_confirm?code=${params.code}&allow=${allow}`,
    {
      res_type: ResponseType.Json,
      content_type: 'x-www-form-urlencoded'
    }
  );

  if (typeof res === 'object') {
    if (res.location !== '') {
      window.location.href = res.location;
    }
    return;
  } else {
    return res;
  }
};

export const test_token = async () => {
  let res = await get<boolean>('/backend/auth/test_token', {
    res_type: ResponseType.Json
  });
  return typeof res === 'boolean' && res;
};
