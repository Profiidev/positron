import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { get_token, TokenType } from "$lib/auth/token.svelte";
import { AuthError, type OAuthParams } from "$lib/auth/types.svelte";

export const auth = async (params: OAuthParams, allow: boolean) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return AuthError.Other;
  }

  try {
    let auth_res = await fetch(
      `${PUBLIC_BACKEND_URL}/oauth/authorize?code=${params.code}&allow=${allow}`,
      {
        method: "POST",
        headers: {
          Authorization: token,
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
