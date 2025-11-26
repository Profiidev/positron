import {
  startAuthentication,
  startRegistration,
  type PublicKeyCredentialCreationOptionsJSON,
  type PublicKeyCredentialRequestOptionsJSON
} from '@simplewebauthn/browser';
import {
  ResponseType,
  RequestError,
  get,
  post
} from 'positron-components/backend';
import type { Passkey } from './types.svelte';

const isKeyCredCreate = (
  object: any
): object is { publicKey: PublicKeyCredentialCreationOptionsJSON } => {
  return typeof object === 'object' && object !== null && 'publicKey' in object;
};

const isKeyCredRequest = (
  object: any
): object is {
  res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
  id: string;
} => {
  return typeof object === 'object' && object !== null && 'res' in object;
};

const isKeyCredRequestSpecial = (
  object: any
): object is {
  publicKey: PublicKeyCredentialRequestOptionsJSON;
} => {
  return typeof object === 'object' && object !== null && 'publicKey' in object;
};

export const passkey_register = async (name: string) => {
  let ret = await get<PublicKeyCredentialCreationOptionsJSON>(
    '/backend/auth/passkey/start_registration',
    ResponseType.Json
  );

  if (!isKeyCredCreate(ret)) {
    return ret;
  }

  let optionsJSON = ret.publicKey;
  let reg;
  try {
    reg = await startRegistration({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/backend/auth/passkey/finish_registration',
    ResponseType.None,
    {
      reg,
      name
    }
  );

  return done;
};

export const passkey_authenticate = async () => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>('/backend/auth/passkey/start_authentication', ResponseType.Json);

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    `/backend/auth/passkey/finish_authentication/${res.id}`,
    ResponseType.None,
    ret
  );

  return done;
};

export const passkey_authenticate_by_email = async (email: string) => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>(`/backend/auth/passkey/start_authentication/${email}`, ResponseType.Json);

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    `/backend/auth/passkey/finish_authentication_by_email/${res.id}`,
    ResponseType.None,
    ret
  );

  return done;
};

export const passkey_special_access = async () => {
  let res = await get<{
    publicKey: PublicKeyCredentialRequestOptionsJSON;
  }>('/backend/auth/passkey/start_special_access', ResponseType.Json);

  if (!isKeyCredRequestSpecial(res)) {
    return res;
  }

  let optionsJSON = res.publicKey;
  let ret;
  try {
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/backend/auth/passkey/finish_special_access',
    ResponseType.None,
    ret
  );

  return done;
};

export const passkey_list = async () => {
  let ret = await get<Passkey[]>(
    '/backend/auth/passkey/list',
    ResponseType.Json
  );
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const passkey_remove = async (name: string) => {
  return await post<undefined>(
    '/backend/auth/passkey/remove',
    ResponseType.None,
    {
      name
    }
  );
};

export const passkey_edit_name = async (name: string, old_name: string) => {
  return await post<undefined>(
    '/backend/auth/passkey/edit_name',
    ResponseType.None,
    {
      name,
      old_name
    }
  );
};
