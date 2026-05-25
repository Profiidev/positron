import z from 'zod';

export const information = z.object({
  name: z.string().min(1, 'Name is required').default(''),
  policies: z.array(z.string()).default([]),
  scope: z.string().min(1, 'Scope is required').default('')
});
