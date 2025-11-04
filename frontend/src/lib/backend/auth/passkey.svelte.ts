import type {
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON
} from '@simplewebauthn/browser';
import { ResponseType, RequestError } from 'positron-components/backend';
import type { Passkey } from './types.svelte';
import { BASE_URL, get, post } from '../util.svelte';
import { PUBLIC_IS_APP } from '$env/static/public';
import { onDestroy, onMount } from 'svelte';
import { toast } from 'positron-components/components/ui';

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
    if (PUBLIC_IS_APP !== 'true') {
      const startRegistration = (await import('@simplewebauthn/browser'))
        .startRegistration;
      reg = await startRegistration({ optionsJSON });
    } else {
      const { register, cancel } = await import('tauri-plugin-webauthn-api');
      await cancel();
      reg = await register(BASE_URL, optionsJSON);
    }
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/auth/passkey/finish_registration',
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
  }>('/auth/passkey/start_authentication', ResponseType.Json);

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    if (PUBLIC_IS_APP !== 'true') {
      const startAuthentication = (await import('@simplewebauthn/browser'))
        .startAuthentication;
      ret = await startAuthentication({ optionsJSON });
    } else {
      const { authenticate, cancel } = await import(
        'tauri-plugin-webauthn-api'
      );
      await cancel();
      ret = await authenticate(BASE_URL, optionsJSON);
    }
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    `/auth/passkey/finish_authentication/${res.id}`,
    ResponseType.None,
    ret
  );

  return done;
};

export const passkey_authenticate_by_email = async (email: string) => {
  let res = await get<{
    res: { publicKey: PublicKeyCredentialRequestOptionsJSON };
    id: string;
  }>(`/auth/passkey/start_authentication/${email}`, ResponseType.Json);

  if (!isKeyCredRequest(res)) {
    return res;
  }

  let optionsJSON = res.res.publicKey;
  let ret;
  try {
    if (PUBLIC_IS_APP !== 'true') {
      const startAuthentication = (await import('@simplewebauthn/browser'))
        .startAuthentication;
      ret = await startAuthentication({ optionsJSON });
    } else {
      const { authenticate, cancel } = await import(
        'tauri-plugin-webauthn-api'
      );
      await cancel();
      ret = await authenticate(BASE_URL, optionsJSON);
    }
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    `/auth/passkey/finish_authentication_by_email/${res.id}`,
    ResponseType.None,
    ret
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
    if (PUBLIC_IS_APP !== 'true') {
      const startAuthentication = (await import('@simplewebauthn/browser'))
        .startAuthentication;
      ret = await startAuthentication({ optionsJSON });
    } else {
      const { authenticate, cancel } = await import(
        'tauri-plugin-webauthn-api'
      );
      await cancel();
      ret = await authenticate(BASE_URL, optionsJSON);
    }
  } catch (_) {
    return RequestError.Unauthorized;
  }

  let done = await post<undefined>(
    '/auth/passkey/finish_special_access',
    ResponseType.None,
    ret
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
  return await post<undefined>('/auth/passkey/remove', ResponseType.None, {
    name
  });
};

export const passkey_edit_name = async (name: string, old_name: string) => {
  return await post<undefined>('/auth/passkey/edit_name', ResponseType.None, {
    name,
    old_name
  });
};

export const createEventListener = (openPinDialog: () => void) => {
  let unregister = () => {};
  let pinDescription = $state('');

  onMount(async () => {
    const { registerListener, WebauthnEventType, PinEventType } = await import(
      'tauri-plugin-webauthn-api'
    );
    unregister = await registerListener((event) => {
      switch (event.type) {
        case WebauthnEventType.PinEvent:
          switch (event.event.type) {
            case PinEventType.InvalidPin:
              openPinDialog();
              pinDescription =
                'Invalid PIN. Please try again.' +
                event.event.attempts_remaining
                  ? ` (${event.event.attempts_remaining} attempts remaining)`
                  : '';
              break;
            case PinEventType.PinRequired:
              openPinDialog();
              pinDescription = 'Please enter your security key PIN to continue';
              break;
            case PinEventType.PinAuthBlocked:
            case PinEventType.PinBlocked:
              toast.error('Security key is blocked. Please contact support.');
              break;
          }
          break;
        case WebauthnEventType.PresenceRequired:
          toast.info('Please touch your security key to continue');
          break;
        case WebauthnEventType.SelectDevice:
          toast.info('Please select your security key by touching it');
          break;
        case WebauthnEventType.SelectKey:
          toast.error('Not Implemented');
          break;
      }
    });
  });

  onDestroy(unregister);

  return {
    pinDescription: () => pinDescription
  };
};
