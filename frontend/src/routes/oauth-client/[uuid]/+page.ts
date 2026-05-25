import type { PageLoad } from './$types';
import {
  infoOauthClient,
  listGroupsOAuthClient,
  listScopesOAuthClient,
  listUsersOAuthClient,
  siteUrl
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
  const sitePromise = siteUrl({ fetch }).then(({ data }) => data);

  return {
    clientRes: resPromise,
    groupsPromise,
    scopesPromise,
    sitePromise,
    usersPromise,
    uuid: params.uuid
  };
};
