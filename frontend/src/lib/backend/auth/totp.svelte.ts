import { ContentType, ResponseType } from 'positron-components/backend';
import type { TotpCode } from './types.svelte';
import { get, post } from '../util.svelte';

export const is_code = (object: any): object is TotpCode => {
  return typeof object === 'object' && object !== null && 'qr' in object;
};

export const totp_get_setup_code = async () => {
  return await get<TotpCode>('/auth/totp/start_setup', ResponseType.Json);
};

export const totp_confirm_setup = async (code: string) => {
  return await post<undefined>(
    '/auth/totp/finish_setup',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      code
    })
  );
};

export const totp_confirm = async (code: string) => {
  return await post<undefined>(
    '/auth/totp/confirm',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      code
    })
  );
};

export const totp_remove = async () => {
  return await post<undefined>(
    '/auth/totp/remove',
    ResponseType.None,
    ContentType.Json,
    undefined
  );
};
