import { ResponseType, get, post } from 'positron-components/backend';
import type { Group } from './types.svelte';

export const list_groups = async () => {
  let ret = await get<Group[]>('/backend/management/group/list', ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_group = async (group: Group) => {
  return await post<undefined>(
    '/backend/management/group/edit',
    ResponseType.None,
    group
  );
};

export const create_group = async (name: string, access_level: number) => {
  return await post<undefined>('/backend/management/group/create', ResponseType.None, {
    name,
    access_level
  });
};

export const delete_group = async (group: string) => {
  return await post<undefined>('/backend/management/group/delete', ResponseType.None, {
    uuid: group
  });
};
