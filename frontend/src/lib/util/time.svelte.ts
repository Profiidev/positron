import { browser } from "$app/environment";
import type { DateTime as DateTimeT } from "luxon";

export let DateTime: typeof DateTimeT;

if (browser) {
  import("luxon").then((luxon) => (DateTime = luxon.DateTime));
}
