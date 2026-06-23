import type JSEncrypt from 'jsencrypt';
import { RequestError } from '@profidev/pleiades/backend';
import { browser } from '$app/environment';
import { key as getKey } from '$lib/client';

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => encrypt;

// oxlint-disable-next-line complexity
export const getSessionMeta = (): SessionMeta => {
  const ua = navigator.userAgent;
  let browser_name = 'Unknown Browser';
  let browser_version: undefined | string = undefined;
  let os = 'Unknown OS';

  if (navigator.userAgentData) {
    const { brands } = navigator.userAgentData;
    const activeBrand = brands.find(
      (b) => b.brand !== 'Chromium' && !b.brand.includes('Not')
    );

    if (activeBrand) {
      browser_name = activeBrand.brand;
      browser_version = activeBrand.version;
    } else if (brands.length > 0) {
      browser_name = brands[0].brand;
      browser_version = brands[0].version;
    }
    os = navigator.userAgentData.platform;

    const application = browser_version
      ? `${browser_name} ${browser_version}`
      : browser_name;

    return {
      application,
      name: `${browser_name} ${os}`,
      operating_system: os
    };
  }

  // -- Detect OS --
  if (ua.includes('Win')) {
    os = 'Windows';
  } else if (ua.includes('Mac')) {
    os = 'MacOS';
  } else if (ua.includes('X11')) {
    os = 'UNIX';
  } else if (ua.includes('Linux')) {
    os = 'Linux';
  } else if (/Android/.test(ua)) {
    os = 'Android';
  } else if (/iPhone|iPad|iPod/.test(ua)) {
    os = 'iOS';
  }

  // -- Detect Browser & Version --
  let match: RegExpExecArray | undefined = undefined;
  if ((match = /Firefox\/(?<version>\d+(?:\.\d+)*)/.exec(ua) ?? undefined)) {
    browser_name = 'Firefox';
    browser_version = match.groups?.version;
  } else if (
    (match = /EDG\/(?<version>\d+(?:\.\d+)*)/i.exec(ua) ?? undefined)
  ) {
    browser_name = 'Edge';
    browser_version = match.groups?.version;
  } else if (
    (match = /Chrome\/(?<version>\d+(?:\.\d+)*)/.exec(ua) ?? undefined)
  ) {
    browser_name = 'Chrome';
    browser_version = match.groups?.version;
  } else if (
    (match = /Safari\/(?<version>\d+(?:\.\d+)*)/.exec(ua) ?? undefined) &&
    !ua.includes('Chrome')
  ) {
    browser_name = 'Safari';
    // For Safari, the real version is usually tied to the "Version/X.X" token
    const versionMatch = /Version\/(?<version>\d+(?:\.\d+)*)/.exec(ua);
    if (versionMatch?.groups?.version) {
      browser_version = versionMatch.groups.version;
    } else {
      browser_version = match.groups?.version;
    }
  } else if (
    (match = /MSIE\s(?<version>\d+(?:\.\d+)*)/.exec(ua) ?? undefined) ||
    ua.includes('Trident/')
  ) {
    browser_name = 'Internet Explorer';
    const rvMatch = /rv:(?<version>\d+(?:\.\d+)*)/.exec(ua);
    if (rvMatch?.groups?.version) {
      browser_version = rvMatch.groups.version;
    } else if (match?.groups?.version) {
      browser_version = match.groups.version;
    } else {
      browser_version = 'Unknown';
    }
  }

  const application = browser_version
    ? `${browser_name} ${browser_version}`
    : browser_name;

  return {
    application,
    name: `${browser_name} ${os}`,
    operating_system: os
  };
};

export interface SessionMeta {
  name: string;
  application: string;
  operating_system: string;
}

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

const URL_SAFE_CHARS =
  'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';

export const generateCodeVerifier = (length = 64) => {
  const randomValues = new Uint32Array(length);
  // Crypto.getRandomValues() gives us cryptographically secure random numbers
  crypto.getRandomValues(randomValues);

  let result = '';
  for (let i = 0; i < length; i += 1) {
    // Map the random number to an index in our character string
    const randomIndex = randomValues[i] % URL_SAFE_CHARS.length;
    result += URL_SAFE_CHARS[randomIndex];
  }

  return result;
};

export const generateCodeChallenge = async (codeVerifier: string) => {
  // 1. Convert the string to ASCII bytes (TextEncoder handles this)
  const encoder = new TextEncoder();
  const data = encoder.encode(codeVerifier);

  // 2. Hash the bytes using SHA-256
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);

  // 3. Convert the ArrayBuffer to a Base64URL string (No Padding)
  const hashArray = new Uint8Array(hashBuffer);
  const base64 = btoa(String.fromCharCode(...hashArray));

  // Make the base64 string URL-safe and remove padding (`=`)
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/[=]+$/, '');
};
