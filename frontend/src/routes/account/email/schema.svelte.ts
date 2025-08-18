import { z } from 'zod';

export const confirmSchema = z.object({
  password: z.string().min(1, 'Password is required')
});

export const emailChange = z
  .object({
    email_input: z.boolean().default(true),
    old_code: z.string().default(''),
    new_code: z.string().default(''),
    email: z.email().default('')
  })
  .superRefine((val, ctx) => {
    if (!val.email_input) {
      if (!val.new_code || val.new_code.length !== 6) {
        ctx.addIssue({
          code: 'custom',
          path: ['new_code'],
          message: 'Code must be 6 characters long'
        });
      }

      if (!val.old_code || val.old_code.length !== 6) {
        ctx.addIssue({
          code: 'custom',
          path: ['old_code'],
          message: 'Code must be 6 characters long'
        });
      }
    }
  });
