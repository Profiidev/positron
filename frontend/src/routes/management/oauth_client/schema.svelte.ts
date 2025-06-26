import { isUrl } from 'positron-components/util';
import { z } from 'zod';

export const createSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  redirect_uri: z.string().url(),
  confidential: z.boolean().default(true),
  additional_redirect_uris: z
    .preprocess((arg: any) => {
      if (typeof arg === 'string') {
        return arg
          .split(' ')
          .map((uri) => uri.trim())
          .filter((uri) => uri !== '');
      }
      return arg;
    }, z.array(z.string().url()).default([]))
    .refine((uris) => uris.every(isUrl), {
      message: 'Not all additional redirect URIs are valid URLs',
    })
});

export const editSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  redirect_uri: z.string().url(),
  additional_redirect_uris: z.preprocess((arg: any) => {
    if (typeof arg === 'string') {
      return arg
        .split(' ')
        .map((uri) => uri.trim())
        .filter((uri) => uri !== '');
    }
    return arg;
  }, z.array(z.string().url()).default([])),
  phantom: z.string().default('').optional()
});

export const deleteSchema = z.object({});

export type CreateSchemaType = typeof createSchema;
export type EditSchemaType = typeof editSchema;
export type DeleteSchemaType = typeof deleteSchema;
