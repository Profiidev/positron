import z from 'zod';

export const instanceUrl = z.object({
  url: z.string().url()
});
