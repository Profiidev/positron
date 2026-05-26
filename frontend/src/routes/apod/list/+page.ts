import { listApod } from '$lib/client';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => {
  const res = listApod({ fetch }).then(({ data }) => data);

  return {
    apodList: res
  };
};
