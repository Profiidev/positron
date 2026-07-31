import {
  HorizontalLayout,
  VerticalLayout
} from '$lib/commands/settings.svelte';
import z from 'zod';

export const settings = z.object({
  horizontal_layout: z.array(z.enum(HorizontalLayout)),
  vertical_layout: z.array(z.enum(VerticalLayout))
});
