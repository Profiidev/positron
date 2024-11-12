import { get_uuid } from "$lib/backend/auth/token.svelte";
import type { Permission } from "../management/types.svelte";
import { info } from "./general.svelte";
import { list, priority as priorityUpdate } from "./permissions.svelte";
import type { UserInfo } from "./types.svelte";

let infoData: UserInfo | undefined = $state();
let permissions: Permission[] | undefined = $state();
let priority: number | undefined = $state();

export const updateInfo = async () => {
  let uuid = get_uuid();
  if (uuid) {
    infoData = await info(uuid);
  }
};

export const getInfo = () => {
  return infoData;
};

export const updatePermissions = async () => {
  permissions = await list();
};

export const getPermissions = () => {
  return permissions;
};

export const updatePriority = async () => {
  priority = await priorityUpdate();
};

export const getPriority = () => {
  return priority;
};

updateInfo();
updatePermissions();
updatePriority();
