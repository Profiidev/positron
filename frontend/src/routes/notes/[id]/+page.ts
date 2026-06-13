import { infoNote, listUsersNote } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, fetch }) => ({
  id: params.id,
  noteRes: infoNote({
    fetch,
    path: { uuid: params.id }
  }),
  usersPromise: listUsersNote({ fetch })
});
