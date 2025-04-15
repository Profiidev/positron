import {
  ContentType,
  get,
  post,
  ResponseType
} from 'positron-components/backend';
import type { Settings } from './types.svelte';

export const user_settings = async () => {
  let ret = await get<Settings>('/account/settings/get', ResponseType.Json);
  if (typeof ret === 'object') {
    return ret;
  }
};

export const user_settings_update = async (settings: Settings) => {
  return await post<undefined>(
    '/account/settings/update',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(settings)
  );
};
