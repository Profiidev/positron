import { infoNote, listNoteSnapshots, listUsersNote } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, fetch, url }) => ({
  error: url.searchParams.get('error'),
  id: params.id,
  noteRes: infoNote({
    fetch,
    path: { uuid: params.id }
  }),
  snapshotsPromise: listNoteSnapshots({
    fetch,
    path: { note_uuid: params.id }
  }).then(({ data }) => data),
  usersPromise: listUsersNote({ fetch })
});
