import { ResponseType, get, post } from 'positron-components/backend';
import type { Group } from './types.svelte';

export const list_groups = async () => {
  let ret = await get<Group[]>('/backend/management/group/list', {
    res_type: ResponseType.Json
  });
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_group = async (group: Group) => {
  return await post('/backend/management/group/edit', { body: group });
};

export const create_group = async (name: string, access_level: number) => {
  return await post('/backend/management/group/create', {
    body: {
      name,
      access_level
    }
  });
};

export const delete_group = async (group: string) => {
  return await post('/backend/management/group/delete', {
    body: {
      uuid: group
    }
  });
};
