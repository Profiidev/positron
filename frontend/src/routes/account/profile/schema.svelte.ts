import { z } from 'zod';

export const profileSchema = z.object({
  name: z.string().min(1, 'Username is required')
});
