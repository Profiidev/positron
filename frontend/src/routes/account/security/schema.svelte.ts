import { z } from "zod";

export const passkeyCreateSchema = z.object({
  name: z.string().min(1, "Name is required"),
});

export const passkeyEditSchema = z.object({
  name: z.string().min(1, "Name is required"),
  phantom: z.string().default("").optional(),
});

export const passkeyDeleteSchema = z.object({});

export type PasskeyCreateSchemaType = typeof passkeyCreateSchema;
export type PasskeyEditSchemaType = typeof passkeyEditSchema;
export type PasskeyDeleteSchemaType = typeof passkeyDeleteSchema;
