import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { EmailError } from "./types.svelte";

export const start_change = async (new_email: string) => {
  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/email/manage/start_change`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
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
  try {
    let res = await fetch(`${PUBLIC_BACKEND_URL}/email/manage/finish_change`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
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
