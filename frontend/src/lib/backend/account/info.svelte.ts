import { profile_info, user_info } from "./general.svelte";
import {
  isProfileInfo,
  isUserInfo,
  type ProfileInfo,
  type UserInfo,
} from "./types.svelte";

let profileInfo: ProfileInfo | undefined = $state();
let userInfo: UserInfo | undefined = $state();

export const updateInfo = async () => {
  await updateUserInfo();
  await updateProfileInfo();
};

const updateUserInfo = async () => {
  let ret = await user_info();
  if (isUserInfo(ret)) {
    userInfo = ret;
  }
};

export const getUserInfo = () => {
  return userInfo;
};

const updateProfileInfo = async () => {
  if (userInfo) {
    let ret = await profile_info(userInfo.uuid);
    if (isProfileInfo(ret)) {
      profileInfo = ret;
    }
  }
};

export const getProfileInfo = () => {
  return profileInfo;
};

updateInfo();
