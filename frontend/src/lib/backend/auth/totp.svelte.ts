import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { AuthError, type TotpCode, type TotpInfo } from "./types.svelte";

export const is_code = (object: any): object is TotpCode => {
  return "qr" in object;
};

export const is_info = (object: any): object is TotpInfo => {
  return "enabled" in object;
};

export const get_setup_code = async (): Promise<AuthError | TotpCode> => {
  try {
    let code_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/start_setup`);

    if (code_res.status !== 200) {
      return AuthError.Other;
    }

    let code_json = await code_res.json();

    return code_json as TotpCode;
  } catch (_) {
    return AuthError.Other;
  }
};

export const confirm_setup = async (
  code: string,
): Promise<AuthError | undefined> => {
  try {
    let done = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/finish_setup`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code,
      }),
    });

    if (done.status === 401) {
      return AuthError.Totp;
    }

    if (done.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const confirm = async (code: string): Promise<AuthError | undefined> => {
  try {
    let done = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/confirm`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code,
      }),
    });

    if (done.status === 401) {
      return AuthError.Totp;
    }

    if (done.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};

export const info = async (): Promise<undefined | TotpInfo> => {
  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/info`);

    if (info_res.status !== 200) {
      return;
    }

    let info = await info_res.json();

    return info as TotpInfo;
  } catch (_) {
    return;
  }
};

export const remove = async (): Promise<AuthError | undefined> => {
  try {
    let remove_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/remove`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (remove_res.status !== 200) {
      return AuthError.Other;
    }
  } catch (_) {
    return AuthError.Other;
  }
};
