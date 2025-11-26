import type { OAuthLogout } from '$lib/backend/auth/types.svelte.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => {
  let urlRed = url.searchParams.get('url');
  let name = url.searchParams.get('name');

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
