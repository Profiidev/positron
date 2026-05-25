import { listOAuthScopes } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url }) => {
  const scopes = listOAuthScopes({ fetch }).then(({ data }) => data);
  return {
    error: url.searchParams.get('error'),
    scopes
  };
};
