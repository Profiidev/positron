import { getLocalTimeZone, now, ZonedDateTime } from "@internationalized/date";
import { UpdateType } from "../ws/types.svelte";
import { create_updater } from "../ws/updater.svelte";
import { get_image, get_image_info, list_apods } from "./apod.svelte";
import type { ApodData, ApodInfo } from "./types.svelte";

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

export const apod = create_updater<ApodData | null>(UpdateType.Apod, async () => {
  controller.abort();

  let ret = await get_image_info(apod_date.toDate().toISOString());

  controller = new AbortController();
  get_image(apod_date.toDate().toISOString(), controller.signal).then(
    (res) => (apod_image = res?.image),
  );
  return ret;
});
let controller = new AbortController();
let apod_image: undefined | string = $state();

export const getApodImage = () => {
  return apod_image;
};

$effect.root(() => {
  $effect(() => {
    if (apod_date) {
      apod.update();
    }
  });
});
