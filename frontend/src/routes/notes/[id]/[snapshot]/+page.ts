import { infoNoteSnapshot } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, fetch }) => ({
  id: params.id,
  snapshot: params.snapshot,
  snapshotRes: infoNoteSnapshot({
    fetch,
    path: { snapshot_id: params.snapshot }
  })
});
