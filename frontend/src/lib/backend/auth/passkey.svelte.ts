import type {
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON
} from '@simplewebauthn/browser';
import {
  get,
  post,
  ContentType,
  ResponseType,
  RequestError
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
    '/auth/passkey/start_registration',
    ResponseType.Json
  );

  if (!isKeyCredCreate(ret)) {
    return ret;
  }

  let optionsJSON = ret.publicKey;
  let reg;
  try {
    const startRegistration = (await import('@simplewebauthn/browser'))
      .startRegistration;
    reg = await startRegistration({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/auth/passkey/finish_registration',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      reg,
      name
    })
  );

  return done;
};

export const passkey_authenticate = async () => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>('/auth/passkey/start_authentication', ResponseType.Json);

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    const startAuthentication = (await import('@simplewebauthn/browser'))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    `/auth/passkey/finish_authentication/${res.id}`,
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(ret)
  );

  return done;
};

export const passkey_special_access = async () => {
  let res = await get<{
    publicKey: PublicKeyCredentialRequestOptionsJSON;
  }>('/auth/passkey/start_special_access', ResponseType.Json);

  if (!isKeyCredRequestSpecial(res)) {
    return res;
  }

  let optionsJSON = res.publicKey;
  let ret;
  try {
    const startAuthentication = (await import('@simplewebauthn/browser'))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/auth/passkey/finish_special_access',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(ret)
  );

  return done;
};

export const passkey_list = async () => {
  let ret = await get<Passkey[]>('/auth/passkey/list', ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const passkey_remove = async (name: string) => {
  return await post<undefined>(
    '/auth/passkey/remove',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name
    })
  );
};

export const passkey_edit_name = async (name: string, old_name: string) => {
  return await post<undefined>(
    '/auth/passkey/edit_name',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name,
      old_name
    })
  );
};
