import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod4 } from 'sveltekit-superforms/adapters';
import { profileSchema } from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    profile: await superValidate(zod4(profileSchema))
  };
};
