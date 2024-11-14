import type { OAuthParams } from "$lib/backend/auth/types.svelte.js";
import { redirect } from "@sveltejs/kit";

export const load = ({ cookies, url }) => {
  let cookie = cookies.get("token");
  let code = url.searchParams.get("code");
  let name = url.searchParams.get("name");

  if (cookie) {
    if (code && name) {
      redirect(302, `/oauth?code=${code}&name=${name}`);
    } else {
      redirect(302, "/");
    }
  } else if (code && name) {
    return {
      oauth_params: {
        code,
        name,
      } as OAuthParams,
    };
  }
  return {};
};
