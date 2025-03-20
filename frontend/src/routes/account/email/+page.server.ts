import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import { confirmSchema, emailChange } from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    emailChange: await superValidate(zod(emailChange)),
    confirm: await superValidate(zod(confirmSchema))
  };
};
