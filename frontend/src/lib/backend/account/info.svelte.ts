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
  let user = await updateUserInfo();
  let profile: ProfileInfo | undefined;

  if (user) {
    let ret = await profile_info(user.uuid);
    if (isProfileInfo(ret)) {
      profile = ret;
    }
  }

  if (user && profile) {
    return [user, profile] as [UserInfo, ProfileInfo];
  }
};

export const userData = create_updater<[UserInfo, ProfileInfo]>(
  UpdateType.User,
  updateProfileInfo,
);
