import { listScopesOAuthClient } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => {
  const scopes = listScopesOAuthClient({ fetch }).then(({ data }) => data);
  return {
    scopes
  };
};
