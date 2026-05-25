import { listOauthClients } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url }) => {
  const clients = listOauthClients({ fetch }).then(({ data }) => data);
  return {
    clients,
    error: url.searchParams.get('error')
  };
};
