import { PUBLIC_BACKEND_URL } from "$env/static/public"
import { startAuthentication, startRegistration } from "@simplewebauthn/browser";
import type { PublicKeyCredentialCreationOptionsJSON, PublicKeyCredentialRequestOptionsJSON } from "@simplewebauthn/types";

export const register = async (email: string): Promise<boolean> => {
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_registration/${email}`);
    let optionsJSON = (await start.json()).publicKey as PublicKeyCredentialCreationOptionsJSON;

    let ret = await startRegistration({ optionsJSON });

    let ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_registration/${email}`, {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(ret),
    });

    return ver.status === 200;
  } catch (_) {
    return false;
  }
}

export const authenticate = async (): Promise<boolean> => {
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_authentication`);
    let start_json = await start.json();
    let optionsJSON = start_json.res.publicKey as PublicKeyCredentialRequestOptionsJSON;

    let ret = await startAuthentication({ optionsJSON });

    const ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_authentication/${start_json.id}`, {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(ret),
    });

    return ver.status === 200;
  } catch (_) {
    return false;
  }
}