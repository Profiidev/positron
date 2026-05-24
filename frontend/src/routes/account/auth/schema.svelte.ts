import { z } from 'zod';

export const passkeyCreateSchema = z.object({
  name: z.string().min(1, 'Name is required')
});

export const passkeyEditSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  phantom: z.string().default('').optional()
});

export const passwordChange = z.object({
  password: z.string().min(1, 'Password is required'),
  password_confirm: z.string().min(1, 'Password Confirm is required')
});

export const totpAdd = z.object({
  code: z.string().min(6, 'Code must be 6 characters long')
});

export const totpRemove = z.object({
  phantom: z.string().default('').optional()
});

export const passkeyDeleteSchema = z.object({});

export const emailChangeSchema = z
  .object({
    email: z.email().default(''),
    email_input: z.boolean().default(true),
    new_code: z.string().default(''),
    old_code: z.string().default('')
  })
  .superRefine((val, ctx) => {
    if (!val.email_input) {
      if (!val.new_code || val.new_code.length !== 6) {
        ctx.addIssue({
          code: 'custom',
          message: 'Code must be 6 characters long',
          path: ['new_code']
        });
      }

      if (!val.old_code || val.old_code.length !== 6) {
        ctx.addIssue({
          code: 'custom',
          message: 'Code must be 6 characters long',
          path: ['old_code']
        });
      }
    }
  });
