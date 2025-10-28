import { z } from 'zod';

export const loginSchema = z
  .object({
    passkey_email_input: z.boolean().default(false),
    code_input: z.boolean().default(false),
    passkey_email: z.email().default('test@example.com'),
    email: z.email().default(''),
    password: z.string().default(''),
    totp: z.string().default('')
  })
  .superRefine((val, ctx) => {
    if (val.code_input) {
      if (!val.totp || val.totp.length !== 6) {
        ctx.addIssue({
          code: 'custom',
          path: ['totp'],
          message: 'Code must be 6 characters long'
        });
      }
    } else {
      if (!val.password || val.password === '') {
        ctx.addIssue({
          code: 'custom',
          path: ['password'],
          message: 'Password is required'
        });
      }
    }
  });

export const pin = z.object({
  pin: z.string().min(1, 'Pin is required')
});
