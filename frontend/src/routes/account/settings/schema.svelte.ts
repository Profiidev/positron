import { z } from 'zod';

export const generalSettings = z.object({
  o_auth_instant_confirm: z.boolean().default(false)
});
