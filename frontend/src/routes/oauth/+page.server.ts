import type { OAuthParams } from '$lib/backend/auth/types.svelte.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => {
  let code = url.searchParams.get('code');
  let name = url.searchParams.get('name');

  let oauth_params: OAuthParams | undefined;
  if (code && name) {
    oauth_params = {
      code,
      name
    };
  }

  return {
    oauth_params
  };
};
