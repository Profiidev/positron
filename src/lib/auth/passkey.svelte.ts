import { PUBLIC_BACKEND_URL } from "$env/static/public"
import { startAuthentication, startRegistration } from "@simplewebauthn/browser";
import type { PublicKeyCredentialCreationOptionsJSON, PublicKeyCredentialRequestOptionsJSON } from "@simplewebauthn/types";
import { get_token, set_token, TokenType } from "./token.svelte";
import { AuthError } from "./error.svelte";

export const register = async (): Promise<AuthError | undefined> => {
  let token = get_token(TokenType.SpecialAccess);
  if (!token) {
    return AuthError.MissingToken;
  }

  let optionsJSON: PublicKeyCredentialCreationOptionsJSON;
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_registration`, {
      headers: {
        Authorization: token,
      },
    });
    optionsJSON = (await start.json()).publicKey as PublicKeyCredentialCreationOptionsJSON;
  } catch (_) {
    return AuthError.Other;
  }

  let ret;
  try {
    ret = await startRegistration({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_registration`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify(ret),
    });

    if (ver.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
}

export const authenticate = async (): Promise<AuthError | undefined> => {
  let start_json;
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_authentication`);
    start_json = await start.json();
  } catch (_) {
    return AuthError.Other;
  }

  let optionsJSON = start_json.res.publicKey as PublicKeyCredentialRequestOptionsJSON;

  let ret;
  try {
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_authentication/${start_json.id}`, {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(ret),
    });
    let token = await ver.text();
    set_token(token, TokenType.Auth);
  } catch (_) {
    return AuthError.Other;
  }
}

export const special_access = async (): Promise<AuthError | undefined> => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return AuthError.MissingToken;
  }

  let optionsJSON: PublicKeyCredentialRequestOptionsJSON;
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_special_access`, {
      headers: {
        Authorization: token,
      },
    });
    optionsJSON = (await start.json()).publicKey as PublicKeyCredentialCreationOptionsJSON;
  } catch (_) {
    return AuthError.Other;
  }

  let ret;
  try {
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_special_access`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify(ret),
    });
    let special_access = await ver.text();
    set_token(special_access, TokenType.SpecialAccess);
  } catch (_) {
    return AuthError.Other;
  }
}
