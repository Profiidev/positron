import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { AuthError, type OAuthParams } from "$lib/backend/auth/types.svelte";

export const auth = async (params: OAuthParams, allow: boolean) => {
  try {
    let auth_res = await fetch(
      `${PUBLIC_BACKEND_URL}/oauth/authorize?code=${params.code}&allow=${allow}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "x-www-form-urlencoded",
        },
      },
    );

    if (auth_res.status === 409) {
      return AuthError.Password;
    }

    if (auth_res.status !== 200) {
      return AuthError.Other;
    }

    let location = await auth_res.text();
    if (location) {
      if (location !== "") {
        window.location.href = location;
      }
      return;
    }

    return AuthError.Other;
  } catch (_) {
    return AuthError.Other;
  }
};
