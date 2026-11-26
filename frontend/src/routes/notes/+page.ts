import { listNotes, notesConfig } from '#lib/client/index.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url }) => ({
  error: url.searchParams.get('error'),
  notes: listNotes({ fetch }).then(({ data }) => data ?? []),
  notesConfig: notesConfig({ fetch }).then(({ data }) => data)
});
