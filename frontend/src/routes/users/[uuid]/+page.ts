import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import {
  listCachesSimple,
  listGroupsSimple,
  mailActive,
  userInfoDetail
} from '$lib/client';

export const load: PageLoad = async ({ params, fetch }) => {
  const resPromise = userInfoDetail({
    fetch,
    path: { uuid: params.uuid }
  });
  const groupsPromise = listGroupsSimple({
    fetch
  });
  const cachesPromise = listCachesSimple({ fetch });
  const mailPromise = mailActive({ fetch });

  const [res, groups, caches, mail] = await Promise.all([
    resPromise,
    groupsPromise,
    cachesPromise,
    mailPromise
  ]);

  if (!res.data) {
    if (res.response.status === 404) {
      redirect(307, '/users?error=user_not_found');
    } else {
      redirect(307, '/users?error=user_other');
    }
  }

  return {
    caches: caches.data,
    groups: groups.data,
    mailActive: mail.data?.active ?? false,
    userInfo: res.data,
    uuid: params.uuid
  };
};
