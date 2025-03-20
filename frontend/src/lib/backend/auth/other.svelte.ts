import {
  get,
  post,
  ContentType,
  ResponseType
} from 'positron-components/backend';
import type { OAuthParams } from './types.svelte';

export const logout = async () => {
  return await post<undefined>(
    '/auth/logout',
    ResponseType.None,
    ContentType.Json,
    undefined
  );
};

export const oauth_auth = async (params: OAuthParams, allow: boolean) => {
  let res = await post<{ location: string }>(
    `/oauth/authorize_confirm?code=${params.code}&allow=${allow}`,
    ResponseType.Json,
    ContentType.UrlFrom,
    undefined
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
  let res = await get<boolean>('/auth/test_token', ResponseType.Json);
  return typeof res === 'boolean' && res;
};
