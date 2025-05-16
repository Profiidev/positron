import { z } from 'zod';

const email = z.string().email();

const isEmail = (test: string) => {
  try {
    let _ = email.parse(test);
    return true;
  } catch (_) {
    return false;
  }
};

export const loginSchema = z
  .object({
    passkey_email_input: z.boolean().default(false),
    code_input: z.boolean().default(false),
    passkey_email: z.string().default(''),
    email: z.string().default(''),
    password: z.string().default(''),
    totp: z.string().default('')
  })
  .superRefine((val, ctx) => {
    if (val.code_input) {
      if (!val.totp || val.totp.length !== 6) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ['totp'],
          message: 'Code must be 6 characters long'
        });
      }
    } else if (val.passkey_email_input) {
      if (!val.passkey_email || !isEmail(val.passkey_email)) {
        ctx.addIssue({
          code: z.ZodIssueCode.invalid_string,
          path: ['passkey_email'],
          validation: 'email'
        });
      }
    } else {
      if (!val.email || !isEmail(val.email)) {
        ctx.addIssue({
          code: z.ZodIssueCode.invalid_string,
          path: ['email'],
          validation: 'email'
        });
      }

      if (!val.password || val.password === '') {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ['password'],
          message: 'Password is required'
        });
      }
    }
  });

export const pin = z.object({
  pin: z.string().min(1, 'Pin is required')
});
