import { listNotes } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url }) => ({
  error: url.searchParams.get('error'),
  notes: listNotes({ fetch }).then(({ data }) => data ?? [])
});
