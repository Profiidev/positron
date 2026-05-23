import type { LayoutLoad } from './$types';
import { info, isSetup, type UserInfo } from '$lib/client';

export const load: LayoutLoad = ({ fetch }) => {
  const setupStatus = isSetup({ fetch });
  const user: Promise<UserInfo> = info({ fetch }).then(
    ({ data }) =>
      data ?? {
        uuid: '',
        name: 'Unknown User',
        email: 'unknown@example.com',
        permissions: [],
        totp_enabled: false
      }
  );

  return {
    setupStatus,
    user
  };
};
