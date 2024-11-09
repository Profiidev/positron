import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { get_token, TokenType } from "$lib/auth/token.svelte";
import type { OAuthParams } from "$lib/auth/types.svelte";

export const auth = async (params: OAuthParams) => {
  let token = get_token(TokenType.Auth);
  if (!token) {
    return;
  }

  try {
    let auth_res = await fetch(
      `${PUBLIC_BACKEND_URL}/oauth/authorize?response_type=${params.response_type}&client_id=${params.client_id}&redirect_uri=${params.redirect_uri}${params.state ? `&state=${params.state}` : ""}&allow=true`,
      {
        method: "POST",
        headers: {
          Authorization: token,
          "Content-Type": "x-www-form-urlencoded",
        },
      },
    );

    if (auth_res.status !== 200) {
      return;
    }

    let location = auth_res.headers.get("X-Location");
    if (location) {
      window.location.href = location;
      return null;
    }
  } catch (_) {
    return;
  }
};
