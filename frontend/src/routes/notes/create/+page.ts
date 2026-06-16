import { listNotes, notesConfig } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => ({
  notes: listNotes({ fetch }).then(({ data }) => data ?? []),
  notesConfig: notesConfig({ fetch }).then(({ data }) => data)
});
