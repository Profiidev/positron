import { PUBLIC_IS_APP } from "$env/static/public";
import type { OAuthParams } from "$lib/backend/auth/types.svelte.js";

export const load = ({ url }) => {
  let code;
  let name;
  if (PUBLIC_IS_APP !== "true") {
    code = url.searchParams.get("code");
    name = url.searchParams.get("name");
  }

  let oauth_params: OAuthParams | undefined;
  if (code && name) {
    oauth_params = {
      code,
      name,
    };
  }

  return {
    oauth_params,
  };
};
