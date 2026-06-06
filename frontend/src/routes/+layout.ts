import type { LayoutLoad } from './$types';
import { type UserInfo, info, isSetup } from '$lib/client';

export const load: LayoutLoad = ({ fetch, url }) => {
  const setupStatus = isSetup({ fetch });
  const user: Promise<UserInfo> = info({ fetch }).then(
    ({ data }) =>
      data ?? {
        email: 'unknown@example.com',
        name: 'Unknown User',
        permissions: [],
        totp_enabled: false,
        uuid: ''
      }
  );

  const code = url.searchParams.get('code');
  const name = url.searchParams.get('name');
  const challenge = url.searchParams.get('challenge');
  let auth = url.searchParams.get('auth');

  if (!auth && url.pathname.startsWith('/auth/')) {
    auth = url.pathname.replace('/auth/', '');
  }

  return {
    auth: {
      authType: auth,
      challenge
    },
    oauthOptions: {
      code,
      name
    },
    setupStatus,
    user
  };
};
