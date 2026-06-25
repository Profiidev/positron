import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
  const code = url.searchParams.get('code') ?? undefined;
  const redirect = url.searchParams.get('redirect') ?? undefined;

  return { code, redirect };
};
