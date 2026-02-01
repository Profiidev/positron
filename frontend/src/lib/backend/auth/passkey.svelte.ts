import type {
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON
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
    {
      res_type: ResponseType.Json
    }
  );

  if (!isKeyCredCreate(ret)) {
    return ret;
  }

  let optionsJSON = ret.publicKey;
  let reg;
  try {
    let startRegistration = (await import('@simplewebauthn/browser'))
      .startRegistration;
    reg = await startRegistration({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post('/backend/auth/passkey/finish_registration', {
    body: {
      reg,
      name
    }
  });

  return done;
};

export const passkey_authenticate = async () => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>('/backend/auth/passkey/start_authentication', {
    res_type: ResponseType.Json
  });

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    let startAuthentication = (await import('@simplewebauthn/browser'))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post(
    `/backend/auth/passkey/finish_authentication/${res.id}`,
    {
      body: ret
    }
  );

  return done;
};

export const passkey_authenticate_by_email = async (email: string) => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>(`/backend/auth/passkey/start_authentication/${email}`, {
    res_type: ResponseType.Json
  });

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    let startAuthentication = (await import('@simplewebauthn/browser'))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post(
    `/backend/auth/passkey/finish_authentication_by_email/${res.id}`,
    {
      body: ret
    }
  );

  return done;
};

export const passkey_special_access = async () => {
  let res = await get<{
    publicKey: PublicKeyCredentialRequestOptionsJSON;
  }>('/backend/auth/passkey/start_special_access', {
    res_type: ResponseType.Json
  });

  if (!isKeyCredRequestSpecial(res)) {
    return res;
  }

  let optionsJSON = res.publicKey;
  let ret;
  try {
    let startAuthentication = (await import('@simplewebauthn/browser'))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post('/backend/auth/passkey/finish_special_access', {
    body: ret
  });

  return done;
};

export const passkey_list = async () => {
  let ret = await get<Passkey[]>('/backend/auth/passkey/list', {
    res_type: ResponseType.Json
  });
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const passkey_remove = async (name: string) => {
  return await post('/backend/auth/passkey/remove', {
    body: {
      name
    }
  });
};

export const passkey_edit_name = async (name: string, old_name: string) => {
  return await post('/backend/auth/passkey/edit_name', {
    body: {
      name,
      old_name
    }
  });
};
