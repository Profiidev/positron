import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { AuthError } from "./error.svelte";
import { get_token, set_token, TokenType } from "./token.svelte";

export type TotpCode = {
  qr: string;
  code: string;
}

export const get_setup_code = async (): Promise<AuthError | TotpCode> => {
  let token = get_token(TokenType.SpecialAccess);
  if (!token) {
    return AuthError.MissingToken;
  }

  try {
    let code_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/start_setup`, {
      headers: {
        Authorization: token,
      }
    });
    let code_json = await code_res.json();

    return code_json as TotpCode;
  } catch (_) {
    return AuthError.Other;
  }
}

export const confirm_setup = async (code: string): Promise<AuthError | undefined> => {
  let token = get_token(TokenType.SpecialAccess);
  if (!token) {
    return AuthError.MissingToken;
  }

  try {
    let done = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/finish_setup`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify({
        code
      }),
    });

    if (done.status === 401) {
      return AuthError.Totp;
    }
  } catch (_) {
    return AuthError.Other;
  }
}

export const confirm = async (code: string): Promise<AuthError | undefined> => {
  let token = get_token(TokenType.TotpRequired);
  if (!token) {
    return AuthError.MissingToken;
  }

  try {
    let done = await fetch(`${PUBLIC_BACKEND_URL}/auth/totp/confirm`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify({
        code
      }),
    });

    if (done.status === 401) {
      return AuthError.Totp;
    }

    let auth_token = await done.text();
    set_token(auth_token, TokenType.Auth);
  } catch (_) {
    return AuthError.Other;
  }
}
