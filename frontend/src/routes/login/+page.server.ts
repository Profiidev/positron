import { PUBLIC_IS_APP } from '$env/static/public';
import type { OAuthParams } from '$lib/backend/auth/types.svelte.js';
import { redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { loginSchema, pin } from './schema.svelte.js';

export const load = async ({ cookies, url }) => {
  if (PUBLIC_IS_APP === 'true')
    return {
      loginForm: await superValidate(zod(loginSchema)),
      pin: await superValidate(zod(pin))
    };

  let cookie = cookies.get('token');

  let code = url.searchParams.get('code');
  let name = url.searchParams.get('name');

  if (cookie) {
    if (code && name) {
      redirect(302, `/oauth?code=${code}&name=${name}`);
    } else {
      redirect(302, '/');
    }
  } else if (code && name) {
    return {
      oauth_params: {
        code,
        name
      } as OAuthParams,
      loginForm: await superValidate(zod(loginSchema)),
      pin: await superValidate(zod(pin))
    };
  }
  return {
    loginForm: await superValidate(zod(loginSchema)),
    pin: await superValidate(zod(pin))
  };
};
