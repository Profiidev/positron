import { redirect } from '@sveltejs/kit';

export const load = () => {
  redirect(302, `/apod/${new Date().toISOString().split('T')[0]}`);
};
