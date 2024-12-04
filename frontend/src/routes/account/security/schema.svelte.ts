import { z } from "zod";

export const passkeyCreateSchema = z.object({
  name: z.string().min(1, "Name is required"),
});

export const passkeyEditSchema = z.object({
  name: z.string().min(1, "Name is required"),
  phantom: z.string().default("").optional(),
});

export const passkeyDeleteSchema = z.object({});

export const confirmSchema = z.object({
  password: z.string().min(1, "Password is required"),
});

export const passwordChange = z.object({
  password: z.string().min(1, "Password is required"),
  password_confirm: z.string().min(1, "Password Confirm is required"),
});

export const totpAdd = z.object({
  code: z.string().min(6, "Code must be 6 characters long"),
});

export const totpRemove = z.object({
  phantom: z.string().default("").optional(),
});

export type PasskeyCreateSchemaType = typeof passkeyCreateSchema;
export type PasskeyEditSchemaType = typeof passkeyEditSchema;
export type PasskeyDeleteSchemaType = typeof passkeyDeleteSchema;
