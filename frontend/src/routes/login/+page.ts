import type { PageLoad } from './$types';
import { authConfig } from '$lib/client';

export const load: PageLoad = async ({ fetch, url }) => {
  const error = url.searchParams.get('error') || undefined;
  if (error) {
    return { error };
  }

  const { data: config } = await authConfig({ fetch });
  return { config };
};
