import { z } from "zod";

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
    code_input: z.boolean().default(false),
    email: z.string().default(""),
    password: z.string().default(""),
    totp: z.string().default(""),
  })
  .superRefine((val, ctx) => {
    if (val.code_input) {
      if (!val.totp || val.totp.length !== 6) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["totp"],
          message: "Code must be 6 characters long",
        });
      }
    } else {
      if (!val.email || !isEmail(val.email)) {
        ctx.addIssue({
          code: z.ZodIssueCode.invalid_string,
          path: ["email"],
          validation: "email",
        });
      }

      if (!val.password || val.password === "") {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["password"],
          message: "Password is required",
        });
      }
    }
  });
