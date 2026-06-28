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

let socket: WebSocket | undefined = $state();
let interval = 0;
let skipError = false;

export const appLoginWebsocket = (
  challenge: string,
  onCode: (code: string) => void,
  onError: () => void
) => {
  if (socket) {
    clearInterval(interval);
    socket.close();
  }

  skipError = false;
  socket = new WebSocket(`/api/auth/app/device_login?challenge=${challenge}`);

  // oxlint-disable-next-line prefer-add-event-listener
  socket.onmessage = (event: MessageEvent<string>) => {
    onCode(event.data);
  };

  // oxlint-disable-next-line prefer-add-event-listener
  socket.onerror = () => {
    if (!skipError) {
      onError();
    }
  };

  // oxlint-disable-next-line prefer-add-event-listener
  socket.onclose = () => {
    if (!skipError) {
      onError();
    }
  };

  // oxlint-disable-next-line no-unsafe-type-assertion
  interval = setInterval(() => {
    if (
      !socket ||
      socket.readyState === socket.CLOSING ||
      socket.readyState === socket.CLOSED
    ) {
      clearInterval(interval);
      return;
    }

    socket.send('heartbeat');
  }, 10_000) as unknown as number;
};

export const cancelAppLogin = () => {
  skipError = true;
  clearInterval(interval);
  socket?.close();
};

export const openAppLoginDeepLink = (code: string, redirect: string) => {
  const url = new URL('positron://login');
  url.searchParams.set('code', code);
  url.searchParams.set('redirect', redirect);
  const link = document.createElement('a');
  link.href = url.toString();
  link.click();
};
