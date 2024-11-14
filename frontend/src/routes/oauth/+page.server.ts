import type { OAuthParams } from "$lib/backend/auth/types.svelte.js";

export const load = ({ url }) => {
  let code = url.searchParams.get("code");
  let name = url.searchParams.get("name");

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
