import { listPasskeys } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const { data } = await listPasskeys({ fetch });

  return { passkeys: data };
};
