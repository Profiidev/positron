import type JSEncrypt from 'jsencrypt';
import {
  ContentType,
  ResponseType,
  RequestError
} from 'positron-components/backend';
import { browser } from '$app/environment';
import { get, post } from '../util.svelte';

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => {
  return encrypt;
};

export const fetch_key = async () => {
  if (encrypt === false) {
    return RequestError.Other;
  }

  let key = await get<{ key: string }>('/auth/password/key', ResponseType.Json);

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
    '/auth/password/authenticate',
    ResponseType.Json,
    ContentType.Json,
    JSON.stringify({
      email,
      password: encrypted_password
    })
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
    '/auth/password/special_access',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      password: encrypted_password
    })
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
  let res = await post<undefined>(
    '/auth/password/change',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      password: encrypted_password,
      password_confirm: encrypted_password_confirm
    })
  );

  if (res && res === RequestError.Unauthorized) {
    fetch_key();
  }
  return res;
};
