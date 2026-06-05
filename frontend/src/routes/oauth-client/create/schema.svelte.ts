import z from 'zod';

export const information = z.object({
  confidential: z.boolean().default(true),
  name: z.string().min(1, 'Name is required').default(''),
  redirect_uri: z.url().default(''),
  require_pkce: z.boolean().default(false),
  scope: z.array(z.string()).default([])
});
