import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
  const { url } = await parent();

  if (!url) {
    redirect(302, '/setup');
  }
};
