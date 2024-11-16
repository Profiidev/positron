import { ContentType, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import type { Group, Permission, UserInfo } from "./types.svelte";

export const list_groups = async () => {
  let ret = await get<Group[]>("/management/group/list", ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const list_groups_user = async () => {
  let ret = await get<UserInfo[]>(
    "/management/group/user_list",
    ResponseType.Json,
  );
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const edit_group_users = async (
  group: string,
  user: string,
  add: boolean,
) => {
  return await post<undefined>(
    "/management/group/edit_users",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid: group,
      user,
      add,
    }),
  );
};

export const edit_group_permissions = async (
  group: string,
  permission: Permission,
  add: boolean,
) => {
  return await post<undefined>(
    "/management/group/edit_permissions",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid: group,
      permission,
      add,
    }),
  );
};

export const edit_group_meta = async (
  group: string,
  name: string,
  access_level: number,
) => {
  return await post<undefined>(
    "/management/group/edit_meta",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid: group,
      name,
      access_level,
    }),
  );
};

export const create_group = async (name: string, access_level: number) => {
  return await post<undefined>(
    "/management/group/create",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name,
      access_level,
    }),
  );
};

export const delete_group = async (group: string) => {
  return await post<undefined>(
    "/management/group/delete",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid: group,
    }),
  );
};
