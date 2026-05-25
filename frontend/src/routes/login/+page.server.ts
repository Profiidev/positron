import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ cookies, url }) => {
  const cookie = cookies.get('centaurus_jwt');
  const code = url.searchParams.get('code');
  const name = url.searchParams.get('name');

  if (cookie) {
    if (code && name) {
      redirect(302, `/oauth?code=${code}&name=${name}`);
    }

    if (url.pathname === '/login') {
      redirect(302, '/');
    }
  }
};
