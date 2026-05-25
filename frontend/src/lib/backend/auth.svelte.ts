import type JSEncrypt from 'jsencrypt';
import { RequestError } from '@profidev/pleiades/backend';
import { browser } from '$app/environment';
import { key as getKey } from '$lib/client';

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => encrypt;

export const fetchKey = async () => {
  if (encrypt === false) {
    return RequestError.Other;
  }

  const { data: keyData } = await getKey();
  if (!keyData) {
    return undefined;
  }

  const { JSEncrypt } = await import('jsencrypt');

  encrypt = new JSEncrypt({ default_key_size: '4096' });
  encrypt.setPublicKey(keyData.key);

  return undefined;
};
const _ = fetchKey();
