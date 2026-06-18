import { infoNoteShare } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, fetch }) => ({
  id: params.id,
  noteRes: infoNoteShare({
    fetch,
    path: { uuid: params.id }
  })
});
