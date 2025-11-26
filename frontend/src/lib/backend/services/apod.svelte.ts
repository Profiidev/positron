import {
  ResponseType,
  RequestError,
  get,
  post
} from 'positron-components/backend';
import type { Apod, ApodData, ApodInfo } from './types.svelte';

export const list_apods = async () => {
  let ret = await get<ApodInfo[]>('/services/apod/list', ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const get_image_info = async (date: string) => {
  let ret = await post<ApodData>(
    '/services/apod/get_image_info',
    ResponseType.Json,
    {
      date
    }
  );

  if (typeof ret === 'object') {
    return ret;
  } else if (ret === RequestError.Gone) {
    return null;
  }
};

export const get_image = async (date: string, signal?: AbortSignal) => {
  let ret = await post<Apod>(
    '/services/apod/get_image',
    ResponseType.Json,
    {
      date
    },
    undefined,
    signal
  );

  if (typeof ret === 'object') {
    return ret;
  }
};

export const set_good = async (good: boolean, date: string) => {
  return await post<undefined>('/services/apod/set_good', ResponseType.None, {
    good,
    date
  });
};
