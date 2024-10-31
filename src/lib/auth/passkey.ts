import { PUBLIC_BACKEND_URL } from "$env/static/public"
import { startAuthentication, startRegistration } from "@simplewebauthn/browser";

export const register = async (username: string): Promise<boolean> => {
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_registration/${username}`);
    let optionsJSON = (await start.json()).publicKey;

    let ret = await startRegistration({ optionsJSON });

    let ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_registration/${username}`, {
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

export const authenticate = async (username: string): Promise<boolean> => {
  try {
    let start = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/start_authentication/${username}`);
    let optionsJSON = (await start.json()).publicKey;

    let ret = await startAuthentication({ optionsJSON });

    const ver = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/finish_authentication/${username}`, {
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