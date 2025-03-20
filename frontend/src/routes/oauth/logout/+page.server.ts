import { PUBLIC_IS_APP } from '$env/static/public';
import type { OAuthLogout } from '$lib/backend/auth/types.svelte.js';

export const load = ({ url }) => {
  let urlRed;
  let name;
  if (PUBLIC_IS_APP !== 'true') {
    urlRed = url.searchParams.get('url');
    name = url.searchParams.get('name');
  }

  let oauth_logout: OAuthLogout | undefined;
  if (urlRed && name) {
    oauth_logout = {
      url: urlRed,
      name
    };
  }

  return {
    oauth_logout
  };
};
