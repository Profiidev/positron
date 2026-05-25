import { listOAuthPolicies } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url }) => {
  const policies = listOAuthPolicies({ fetch }).then(({ data }) => data);
  return {
    error: url.searchParams.get('error'),
    policies
  };
};
