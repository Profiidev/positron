import { ResponseType, get, post } from 'positron-components/backend';
import type { TotpCode } from './types.svelte';

export const is_code = (object: any): object is TotpCode => {
  return typeof object === 'object' && object !== null && 'qr' in object;
};

export const totp_get_setup_code = async () => {
  return await get<TotpCode>('/backend/auth/totp/start_setup', {
    res_type: ResponseType.Json
  });
};

export const totp_confirm_setup = async (code: string) => {
  return await post('/backend/auth/totp/finish_setup', {
    body: {
      code
    }
  });
};

export const totp_confirm = async (code: string) => {
  return await post('/backend/auth/totp/confirm', {
    body: {
      code
    }
  });
};

export const totp_remove = async () => {
  return await post('/backend/auth/totp/remove');
};
