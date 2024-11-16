import type JSEncrypt from "jsencrypt";
import { ContentType, RequestError, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import { browser } from "$app/environment";

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => {
  return encrypt;
};

export const fetch_key = async () => {
  if (encrypt === false) {
    return RequestError.Other;
  }

  let key = await get<string>("/auth/password/key", ResponseType.Text);

  if (typeof key !== "string") {
    return key;
  }

  const JSEncrypt = (await import("jsencrypt")).JSEncrypt;

  encrypt = new JSEncrypt({ default_key_size: "4096" });
  encrypt.setPublicKey(key);
};
fetch_key();

export const password_login = async (email: string, password: string) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let res = await post<string>(
    "/auth/password/authenticate",
    ResponseType.Text,
    ContentType.Json,
    JSON.stringify({
      email,
      password: encrypted_password,
    }),
  );

  if (typeof res !== "string") {
    if (res === RequestError.Unauthorized) {
      fetch_key();
    }
    return res;
  }

  return Boolean(res);
};

export const password_special_access = async (password: string) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let res = await post<undefined>(
    "/auth/password/special_access",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      password: encrypted_password,
    }),
  );

  if (res && res === RequestError.Unauthorized) {
    fetch_key();
  }
  return res;
};

export const password_change = async (
  password: string,
  password_confirm: string,
) => {
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(password);
  let encrypted_password_confirm = encrypt.encrypt(password_confirm);
  let res = await post<undefined>(
    "/auth/password/change",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      password: encrypted_password,
      password_confirm: encrypted_password_confirm,
    }),
  );

  if (res && res === RequestError.Unauthorized) {
    fetch_key();
  }
  return res;
};
