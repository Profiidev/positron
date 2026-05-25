import z from 'zod';

export const information = z.object({
  confidential: z.boolean().default(false),
  name: z.string().min(1, 'Name is required').default(''),
  redirect_uri: z.url().default(''),
  scope: z.array(z.string()).default([])
});
