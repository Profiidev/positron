import { getLocalTimeZone, now, ZonedDateTime } from "@internationalized/date";
import { UpdateType } from "../ws/types.svelte";
import { create_updater } from "../ws/updater.svelte";
import { get_image, list_apods } from "./apod.svelte";
import type { Apod, ApodInfo } from "./types.svelte";

export const apod_info_list = create_updater<ApodInfo[]>(
  UpdateType.Apod,
  list_apods,
);

let apod_date = $state(now(getLocalTimeZone()));

export const setApodDate = (date: ZonedDateTime) => {
  apod_date = date;
};

export const getApodDate = () => {
  return apod_date;
};

export const apod = create_updater<Apod>(UpdateType.Apod, async () => {
  return await get_image(apod_date.toDate().toISOString());
});

$effect.root(() => {
  $effect(() => {
    if (apod_date) {
      apod.update();
    }
  });
});
