import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod4 } from 'sveltekit-superforms/adapters';
import { createSchema, deleteSchema, editSchema } from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    createForm: await superValidate(zod4(createSchema)),
    editForm: await superValidate(zod4(editSchema)),
    deleteForm: await superValidate(zod4(deleteSchema))
  };
};
