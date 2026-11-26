import { listScopesOAuthClient } from '#lib/client/index.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => {
  const scopes = listScopesOAuthClient({ fetch }).then(({ data }) => data);
  return {
    scopes
  };
};
