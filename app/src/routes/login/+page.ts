import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
  const code = url.searchParams.get('code') ?? undefined;

  return { code };
};
