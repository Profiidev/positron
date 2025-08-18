import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod4 } from 'sveltekit-superforms/adapters';
import { confirmSchema, emailChange } from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    emailChange: await superValidate(zod4(emailChange)),
    confirm: await superValidate(zod4(confirmSchema))
  };
};
