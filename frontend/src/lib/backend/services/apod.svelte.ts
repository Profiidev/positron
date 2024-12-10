import { ContentType, ResponseType } from "../types.svelte";
import { get, post } from "../util.svelte";
import type { Apod, ApodInfo } from "./types.svelte";

export const list_apods = async () => {
  let ret = await get<ApodInfo[]>("/services/apod/list", ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const get_image = async (date: string) => {
  let ret = await post<Apod>(
    "/services/apod/get_image",
    ResponseType.Json,
    ContentType.Json,
    JSON.stringify({
      date,
    }),
  );

  if (typeof ret === "object") {
    return ret;
  }
};

export const set_good = async (good: boolean, date: string) => {
  return await post<undefined>(
    "/services/apod/set_good",
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      good,
      date,
    }),
  );
};