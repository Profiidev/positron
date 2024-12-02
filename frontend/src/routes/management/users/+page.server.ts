import { superValidate } from "sveltekit-superforms";
import type { PageServerLoad } from "./$types";
import { zod } from "sveltekit-superforms/adapters";
import { createSchema, deleteSchema, editSchema } from "./schema.svelte";

export const load: PageServerLoad = async () => {
  return {
    createForm: await superValidate(zod(createSchema)),
    editForm: await superValidate(zod(editSchema)),
    deleteForm: await superValidate(zod(deleteSchema)),
  };
};
