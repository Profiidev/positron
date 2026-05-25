import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => {
  const urlReq = url.searchParams.get('url');
  const name = url.searchParams.get('name');

  return {
    oauthLogout:
      urlReq && name
        ? {
            name,
            url: urlReq
          }
        : undefined
  };
};
