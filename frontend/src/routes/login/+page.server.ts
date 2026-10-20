import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getRedirectTarget } from '$lib/redirect';

export const load: PageServerLoad = ({ cookies, url }) => {
  const cookie = cookies.get('centaurus_jwt');
  const code = url.searchParams.get('code');
  const name = url.searchParams.get('name');
  const auth = url.searchParams.get('auth');
  const challenge = url.searchParams.get('challenge');

  if (cookie) {
    if (code && name) {
      redirect(302, `/oauth?code=${code}&name=${name}`);
    }

    if (auth) {
      const query = challenge
        ? `?challenge=${encodeURIComponent(challenge)}`
        : '';
      redirect(302, `/auth/${auth}${query}`);
    }

    if (url.pathname === '/login') {
      redirect(302, getRedirectTarget(url.searchParams));
    }
  }
};
