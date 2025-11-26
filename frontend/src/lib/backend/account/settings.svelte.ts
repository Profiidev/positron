import { ResponseType, get, post } from 'positron-components/backend';
import type { Settings } from './types.svelte';
import { create_updater } from '../ws/updater.svelte';
import { UpdateType } from '../ws/types.svelte';

export const user_settings_get = async () => {
  let ret = await get<Settings>('/account/settings/get', ResponseType.Json);
  if (typeof ret === 'object') {
    return ret;
  }
};
export const user_settings = create_updater<Settings>(
  UpdateType.Settings,
  user_settings_get
);

export const user_settings_update = async (settings: Settings) => {
  return await post<undefined>(
    '/account/settings/update',
    ResponseType.None,
    settings
  );
};
