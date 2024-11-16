import type { Permission } from "../management/types.svelte";

export const isProfileInfo = (object: any): object is ProfileInfo => {
  return typeof object === "object" && object !== null && "name" in object;
};

export const isUserInfo = (object: any): object is UserInfo => {
  return typeof object === "object" && object !== null && "uuid" in object;
};

export interface ProfileInfo {
  name: string;
  image: string;
  email: string;
}

export interface UserInfo {
  last_login: string;
  last_special_access: string;
  totp_enabled: boolean;
  totp_created?: string;
  totp_last_used?: string;
  uuid: string;
  permissions: Permission[];
  access_level: number;
}
