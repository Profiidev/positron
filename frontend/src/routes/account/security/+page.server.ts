import { superValidate } from "sveltekit-superforms";
import type { PageServerLoad } from "./$types";
import { zod } from "sveltekit-superforms/adapters";
import {
  passkeyCreateSchema,
  passkeyDeleteSchema,
  passkeyEditSchema,
} from "./schema.svelte";

export const load: PageServerLoad = async () => {
  return {
    passkeyCreateForm: await superValidate(zod(passkeyCreateSchema)),
    passkeyEditForm: await superValidate(zod(passkeyEditSchema)),
    passkeyDeleteForm: await superValidate(zod(passkeyDeleteSchema)),
  };
};
