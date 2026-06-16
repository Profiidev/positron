import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types.js';
import { noAuthPaths } from '$lib/components/nav.svelte.js';

export const load: LayoutServerLoad = ({ cookies, url }) => {
  const cookie = cookies.get('centaurus_jwt');

  if (!cookie && !noAuthPaths.includes(url.pathname)) {
    redirect(302, '/login');
  }
};
