import { base64ToArrayBuffer } from 'positron-components/util/convert.svelte';
import { ResponseType, get, post } from 'positron-components/backend';
import type { ProfileInfo, UserInfo } from './types.svelte';

export const profile_info = async (uuid: string) => {
  return await get<ProfileInfo>(
    `/backend/account/general/profile_info/${uuid}`,
    {
      res_type: ResponseType.Json
    }
  );
};

export const user_info = async () => {
  return await get<UserInfo>('/backend/account/general/info', {
    res_type: ResponseType.Json
  });
};

export const profile_change_image = async (image: string) => {
  return await post('/backend/account/general/change_image', {
    body: base64ToArrayBuffer(image)
  });
};

export const profile_update = async (name: string) => {
  return await post('/backend/account/general/update_profile', {
    body: { name }
  });
};
