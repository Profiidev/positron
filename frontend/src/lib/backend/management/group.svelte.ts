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

export const edit_group = async (group: Group) => {
  return await post<undefined>(
    "/management/group/edit",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify(group),
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
