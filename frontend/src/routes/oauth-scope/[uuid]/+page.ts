import type { PageLoad } from './$types';
import { infoOAuthScope, listPoliciesOAuthScope } from '$lib/client';

export const load: PageLoad = ({ params, fetch }) => {
  const resPromise = infoOAuthScope({
    fetch,
    path: { uuid: params.uuid }
  });

  const policiesPromise = listPoliciesOAuthScope({ fetch }).then(
    ({ data }) => data
  );

  return {
    policiesPromise,
    scopeRes: resPromise,
    uuid: params.uuid
  };
};
