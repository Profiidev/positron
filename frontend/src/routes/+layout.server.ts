import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types.js';

export const load: LayoutServerLoad = ({ cookies, url }) => {
  let cookie = cookies.get('token');

  if (!cookie && url.pathname !== '/login' && url.pathname !== '/oauth') {
    redirect(302, '/login');
  }
};
