import { listPoliciesOAuthScope } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => {
  const policies = listPoliciesOAuthScope({ fetch }).then(({ data }) => data);
  return {
    policies
  };
};
