import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getRedirectTarget } from '$lib/redirect';

export const load: PageServerLoad = ({ cookies, url }) => {
  const cookie = cookies.get('centaurus_jwt');
  const code = url.searchParams.get('code');
  const name = url.searchParams.get('name');
  const auth = url.searchParams.get('auth');

  if (cookie) {
    if (code && name) {
      redirect(302, `/oauth?code=${code}&name=${name}`);
    }

    if (auth) {
      redirect(302, `/auth/${auth}`);
    }

    if (url.pathname === '/login') {
      redirect(302, getRedirectTarget(url.searchParams));
    }
  }
};
