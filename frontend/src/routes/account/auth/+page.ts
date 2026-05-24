import { listPasskeys, mailActive } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const passkeyPromise = listPasskeys({ fetch }).then(({ data }) => data);
  const mailPromise = mailActive({ fetch }).then(
    (res) => res.data?.active ?? false
  );

  return {
    mailActive: mailPromise,
    passkeys: passkeyPromise
  };
};
