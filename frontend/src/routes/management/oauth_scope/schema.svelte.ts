import { z } from "zod";

export const createSchema = z.object({
  name: z.string().min(1, "Name is required"),
  scope: z.string().min(1, "Scope is required"),
});

export const editSchema = z.object({
  name: z.string().min(1, "Name is required"),
  scope: z.string().min(1, "Scope is required"),
  phantom: z.string().default("").optional(),
});

export const deleteSchema = z.object({});

export type CreateSchemaType = typeof createSchema;
export type EditSchemaType = typeof editSchema;
export type DeleteSchemaType = typeof deleteSchema;
