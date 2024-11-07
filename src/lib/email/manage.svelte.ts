import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { get_token, TokenType } from "$lib/auth/token.svelte";
import { EmailError } from "./types.svelte";

export const start_change = async (new_email: string) => {
  let token = get_token(TokenType.SpecialAccess);
  if (!token) {
    return;
  }

  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/email/manage/start_change`, {
      method: "POST",
      headers: {
        Authorization: token,
      },
      body: JSON.stringify({
        new_email,
      }),
    });

    if (res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};

export const finish_change = async (old_code: string, new_code: string) => {
  let token = get_token(TokenType.SpecialAccess);
  if (!token) {
    return EmailError.Other;
  }

  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/email/manage/finish_change`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token,
      },
      body: JSON.stringify({
        old_code,
        new_code,
      }),
    });

    if (res.status === 401) {
      return EmailError.Code;
    }

    if (res.status !== 200) {
      return EmailError.Other;
    }
  } catch (_) {
    return EmailError.Other;
  }
};
