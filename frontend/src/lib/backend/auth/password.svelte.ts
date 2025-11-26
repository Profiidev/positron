import type JSEncrypt from 'jsencrypt';
import {
  ResponseType,
  RequestError,
  get,
  post
} from 'positron-components/backend';
import { browser } from '$app/environment';

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => {
  return encrypt;
};

export const fetch_key = async () => {
  if (encrypt === false) {
    return RequestError.Other;
  }

  let key = await get<{ key: string }>('/backend/auth/password/key', ResponseType.Json);

  if (typeof key !== 'object') {
    return key;
  }

  const JSEncrypt = (await import('jsencrypt')).JSEncrypt;

  encrypt = new JSEncrypt({ default_key_size: '4096' });
  encrypt.setPublicKey(key.key);
};
fetch_key();

export const password_login = async (email: string, password: string) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let res = await post<{ totp: boolean }>(
    '/backend/auth/password/authenticate',
    ResponseType.Json,
    {
      email,
      password: encrypted_password
    }
  );

  if (typeof res !== 'object') {
    if (res === RequestError.Unauthorized) {
      fetch_key();
    }
    return res;
  }

  return res.totp;
};

export const password_special_access = async (password: string) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let res = await post<undefined>(
    '/backend/auth/password/special_access',
    ResponseType.None,
    {
      password: encrypted_password
    }
  );

  if (res && res === RequestError.Unauthorized) {
    fetch_key();
  }
  return res;
};

export const password_change = async (
  password: string,
  password_confirm: string
) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let encrypted_password_confirm = encrypt.encrypt(password_confirm);
  let res = await post<undefined>('/backend/auth/password/change', ResponseType.None, {
    password: encrypted_password,
    password_confirm: encrypted_password_confirm
  });

  if (res && res === RequestError.Unauthorized) {
    fetch_key();
  }
  return res;
};
