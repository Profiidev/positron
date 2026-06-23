import { listSessions } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => ({
  sessions: listSessions({ fetch }).then(({ data }) => data ?? [])
});
