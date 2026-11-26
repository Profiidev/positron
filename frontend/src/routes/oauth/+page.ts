import { accountSettings } from '#lib/client/index.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch }) => {
  const settings = accountSettings({ fetch }).then(({ data }) => data);

  return { settings };
};
