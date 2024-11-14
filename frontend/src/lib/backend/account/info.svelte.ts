import { get_uuid } from "$lib/backend/auth/token.svelte";
import { Permission } from "../management/types.svelte";
import { info } from "./general.svelte";
import { list, access_level as accessLevelUpdate } from "./permissions.svelte";
import type { UserInfo } from "./types.svelte";

let infoData: UserInfo | undefined = $state();
let permissions: Permission[] | undefined = $state();
let access_level: number | undefined = $state();

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

export const updateAccessLevel = async () => {
  access_level = await accessLevelUpdate();
};

export const getAccessLevel = () => {
  return access_level;
};

updateInfo();
updatePermissions();
updateAccessLevel();
