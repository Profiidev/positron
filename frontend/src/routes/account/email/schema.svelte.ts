import { z } from "zod";

export const confirmSchema = z.object({
  password: z.string().min(1, "Password is required"),
});

const email = z.string().email();

const isEmail = (test: string) => {
  try {
    let _ = email.parse(test);
    return true;
  } catch (_) {
    return false;
  }
};

export const emailChange = z
  .object({
    email_input: z.boolean().default(true),
    old_code: z.string().default(""),
    new_code: z.string().default(""),
    email: z.string().default(""),
  })
  .superRefine((val, ctx) => {
    if (val.email_input) {
      if (!val.email || val.email === "" || !isEmail(val.email)) {
        ctx.addIssue({
          code: z.ZodIssueCode.invalid_string,
          path: ["email"],
          validation: "email",
        });
      }
    } else {
      if (!val.new_code || val.new_code.length !== 6) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["new_code"],
          message: "Code must be 6 characters long",
        });
      }

      if (!val.old_code || val.old_code.length !== 6) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["old_code"],
          message: "Code must be 6 characters long",
        });
      }
    }
  });
