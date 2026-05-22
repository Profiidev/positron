import { z } from 'zod';

export const login = z.object({
  email: z.email('Invalid email address'),
  password: z.string().min(1, 'Password is required')
});

export const totpSchema = z.object({
  code: z
    .string()
    .min(6, 'TOTP code must be 6 digits')
    .max(6, 'TOTP code must be 6 digits')
});
