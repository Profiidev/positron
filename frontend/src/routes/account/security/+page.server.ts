import { superValidate } from 'sveltekit-superforms';
import type { PageServerLoad } from './$types';
import { zod4 } from 'sveltekit-superforms/adapters';
import {
  confirmSchema,
  passkeyCreateSchema,
  passkeyDeleteSchema,
  passkeyEditSchema,
  passwordChange,
  pin,
  totpAdd,
  totpRemove
} from './schema.svelte';

export const load: PageServerLoad = async () => {
  return {
    passkeyCreateForm: await superValidate(zod4(passkeyCreateSchema)),
    passkeyEditForm: await superValidate(zod4(passkeyEditSchema)),
    passkeyDeleteForm: await superValidate(zod4(passkeyDeleteSchema)),
    confirmForm: await superValidate(zod4(confirmSchema)),
    passwordChange: await superValidate(zod4(passwordChange)),
    totpRemove: await superValidate(zod4(totpRemove)),
    totpAdd: await superValidate(zod4(totpAdd)),
    pin: await superValidate(zod4(pin))
  };
};
