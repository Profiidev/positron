import {
  HorizontalLayout,
  VerticalLayout
} from '$lib/commands/settings.svelte';
import z from 'zod';

export const settings = z.object({
  height: z.coerce.number().int().min(500),
  horizontal_layout: z.array(z.enum(HorizontalLayout)),
  vertical_layout: z.array(z.enum(VerticalLayout)),
  width: z.coerce.number().int().min(500)
});
