import { get_uuid } from "$lib/backend/auth/token.svelte";
import { info } from "./general.svelte";
import type { UserInfo } from "./types.svelte";

let infoData: UserInfo | undefined = $state();

export const updateInfo = async () => {
  let uuid = get_uuid();
  if (uuid) {
    infoData = await info(uuid);
  }
};

export const getInfo = () => {
  return infoData;
};

updateInfo();
