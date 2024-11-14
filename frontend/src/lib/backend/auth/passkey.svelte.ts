import { PUBLIC_BACKEND_URL } from "$env/static/public";
import type {
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON,
} from "@simplewebauthn/types";
import { AuthError, type Passkey } from "./types.svelte";

export const register = async (
  name: string,
): Promise<AuthError | undefined> => {
  let optionsJSON: PublicKeyCredentialCreationOptionsJSON;
  try {
    let start = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/start_registration`,
    );

    if (start.status !== 200) {
      return AuthError.Other;
    }

    optionsJSON = (await start.json())
      .publicKey as PublicKeyCredentialCreationOptionsJSON;
  } catch (_) {
    return AuthError.Other;
  }

  let ret;
  try {
    const startRegistration = (await import("@simplewebauthn/browser"))
      .startRegistration;
    ret = await startRegistration({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/finish_registration`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          reg: ret,
          name,
        }),
      },
    );

    if (ver.status === 409) {
      return AuthError.Conflict;
    }

    if (ver.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const authenticate = async (): Promise<AuthError | undefined> => {
  let start_json;
  try {
    let start = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/start_authentication`,
    );

    if (start.status !== 200) {
      return AuthError.Other;
    }

    start_json = await start.json();
  } catch (_) {
    return AuthError.Other;
  }

  let optionsJSON = start_json.res
    .publicKey as PublicKeyCredentialRequestOptionsJSON;

  let ret;
  try {
    const startAuthentication = (await import("@simplewebauthn/browser"))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/finish_authentication/${start_json.id}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(ret),
      },
    );

    if (ver.status !== 200) {
      return AuthError.Other;
    }
  } catch (e) {
    return AuthError.Other;
  }
};

export const special_access = async (): Promise<AuthError | undefined> => {
  let optionsJSON: PublicKeyCredentialRequestOptionsJSON;
  try {
    let start = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/start_special_access`,
    );

    if (start.status !== 200) {
      return AuthError.Other;
    }

    optionsJSON = (await start.json())
      .publicKey as PublicKeyCredentialCreationOptionsJSON;
  } catch (_) {
    return AuthError.Other;
  }

  let ret;
  try {
    const startAuthentication = (await import("@simplewebauthn/browser"))
      .startAuthentication;
    ret = await startAuthentication({ optionsJSON });
  } catch (_) {
    return AuthError.Passkey;
  }

  try {
    let ver = await fetch(
      `${PUBLIC_BACKEND_URL}/auth/passkey/finish_special_access`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(ret),
      },
    );

    if (ver.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const list = async () => {
  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/list`);
    let keys = (await res.json()) as Passkey[];
    return keys;
  } catch (_) {
    return;
  }
};

export const remove = async (name: string) => {
  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/remove`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name,
      }),
    });

    if (res.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const edit_name = async (name: string, old_name: string) => {
  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/auth/passkey/edit_name`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name,
        old_name,
      }),
    });

    if (res.status === 409) {
      return AuthError.Conflict;
    }

    if (res.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};
