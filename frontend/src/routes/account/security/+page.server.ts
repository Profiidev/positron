import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import {
  confirmSchema,
  passkeyCreateSchema,
  passkeyDeleteSchema,
  passkeyEditSchema,
  passwordChange,
  totpAdd,
  totpRemove
} from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    passkeyCreateForm: await superValidate(zod(passkeyCreateSchema)),
    passkeyEditForm: await superValidate(zod(passkeyEditSchema)),
    passkeyDeleteForm: await superValidate(zod(passkeyDeleteSchema)),
    confirmForm: await superValidate(zod(confirmSchema)),
    passwordChange: await superValidate(zod(passwordChange)),
    totpRemove: await superValidate(zod(totpRemove)),
    totpAdd: await superValidate(zod(totpAdd))
  };
};
