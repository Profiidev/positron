import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { groupInfo, listCachesSimpleGroup, listUsersSimple } from '$lib/client';

export const load: PageLoad = async ({ params, fetch }) => {
  const resPromise = groupInfo({
    fetch,
    path: { uuid: params.uuid }
  });
  const usersPromise = listUsersSimple({ fetch });
  const cachesPromise = listCachesSimpleGroup({ fetch });

  const [res, users, caches] = await Promise.all([
    resPromise,
    usersPromise,
    cachesPromise
  ]);

  if (!res.data) {
    if (res.response.status === 404) {
      redirect(307, '/groups?error=group_not_found');
    } else {
      redirect(307, '/groups?error=group_other');
    }
  }

  return {
    caches: caches.data,
    group: res.data,
    users: users.data,
    uuid: params.uuid
  };
};
