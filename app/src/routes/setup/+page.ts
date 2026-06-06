import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
  const { isSetup } = await parent();

  if (isSetup) {
    redirect(302, '/');
  }
};
