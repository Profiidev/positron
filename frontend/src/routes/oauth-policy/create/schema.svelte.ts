import z from 'zod';

export const information = z.object({
  claim: z.string().min(1, 'Claim is required').default(''),
  default: z.string().min(1, 'Default is required').default(''),
  name: z.string().min(1, 'Name is required').default('')
});
