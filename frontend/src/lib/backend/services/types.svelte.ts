import type { UserInfo } from "../management/types.svelte";

export interface ApodInfo {
  title: string;
  date: Date;
  image: string;
  user: UserInfo;
}

export interface Apod {
  title: string;
  image: string;
}
