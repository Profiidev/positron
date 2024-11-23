import { UpdateType } from "../ws/types.svelte";
import { create_updater } from "../ws/updater.svelte";
import { profile_info, user_info } from "./general.svelte";
import {
  isProfileInfo,
  isUserInfo,
  type ProfileInfo,
  type UserInfo,
} from "./types.svelte";

const updateUserInfo = async () => {
  let ret = await user_info();
  if (isUserInfo(ret)) {
    return ret;
  }
};

const updateProfileInfo = async () => {
  if (userInfo.value) {
    let ret = await profile_info(userInfo.value.uuid);
    if (isProfileInfo(ret)) {
      return ret;
    }
  }
};

export const userInfo = create_updater<UserInfo>(
  UpdateType.User,
  updateUserInfo,
);

export const profileInfo = create_updater<ProfileInfo>(
  UpdateType.User,
  updateProfileInfo,
);
