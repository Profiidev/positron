import { getApodImageInfo } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
  const res = getApodImageInfo({
    body: {
      date: new Date(params.date)
    },
    fetch
  }).then(({ data, response }) => {
    if (response?.status === 410) {
      // oxlint-disable-next-line no-null
      return null;
    } else {
      return data;
    }
  });

  return {
    apodInfo: res,
    date: params.date
  };
};
