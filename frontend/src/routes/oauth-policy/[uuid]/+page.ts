import type { PageLoad } from './$types';
import { infoOAuthPolicy, listGroupsOAuthPolicy } from '$lib/client';

export const load: PageLoad = ({ params, fetch }) => {
  const resPromise = infoOAuthPolicy({
    fetch,
    path: { uuid: params.uuid }
  });

  const groupsPromise = listGroupsOAuthPolicy({ fetch }).then(
    ({ data }) => data
  );

  return {
    groupsPromise,
    policyRes: resPromise,
    uuid: params.uuid
  };
};
