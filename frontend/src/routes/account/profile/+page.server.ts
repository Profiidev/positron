import { superValidate } from "sveltekit-superforms";
import type { PageServerLoad } from "./$types";
import { zod } from "sveltekit-superforms/adapters";
import { profileSchema } from "./schema.svelte";

export const load: PageServerLoad = async () => {
  return {
    profile: await superValidate(zod(profileSchema)),
  };
};
