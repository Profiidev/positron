import { UpdateType } from "../ws/types.svelte";
import { create_updater } from "../ws/updater.svelte";
import { list_apods } from "./apod.svelte";
import type { ApodInfo } from "./types.svelte";

export const user_info_list = create_updater<ApodInfo[]>(
  UpdateType.Apod,
  list_apods,
);
