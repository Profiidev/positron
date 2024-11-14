import { Permission } from "../management/types.svelte";
import { get_uuid, info } from "./general.svelte";
import { list, access_level as accessLevelUpdate } from "./permissions.svelte";
import type { UserInfo } from "./types.svelte";

let infoData: UserInfo | undefined = $state();
let permissions: Permission[] | undefined = $state();
let access_level: number | undefined = $state();
let uuid: string | undefined = $state();

export const updateInfo = async () => {
  await updateUuid();
  await Promise.all([
    updatePermissions(),
    updateAccessLevel(),
    updateUserInfo(),
  ]);
};

const updateUserInfo = async () => {
  if (uuid) {
    infoData = await info(uuid);
  }
};

export const getInfo = () => {
  return infoData;
};

const updatePermissions = async () => {
  permissions = await list();
};

export const getPermissions = () => {
  return permissions;
};

const updateAccessLevel = async () => {
  access_level = await accessLevelUpdate();
};

export const getAccessLevel = () => {
  return access_level;
};

export const getUuid = () => {
  return uuid;
};

const updateUuid = async () => {
  uuid = await get_uuid();
};

updateInfo();
