import type { PageLoad } from './$types';
import {
  infoOauthClient,
  listGroupsOAuthClient,
  listScopesOAuthClient,
  listUsersOAuthClient
} from '$lib/client';

export const load: PageLoad = ({ params, fetch }) => {
  const resPromise = infoOauthClient({
    fetch,
    path: { uuid: params.uuid }
  });

  const usersPromise = listUsersOAuthClient({ fetch }).then(({ data }) => data);
  const groupsPromise = listGroupsOAuthClient({ fetch }).then(
    ({ data }) => data
  );
  const scopesPromise = listScopesOAuthClient({ fetch }).then(
    ({ data }) => data
  );

  return {
    clientRes: resPromise,
    groupsPromise,
    scopesPromise,
    usersPromise,
    uuid: params.uuid
  };
};
