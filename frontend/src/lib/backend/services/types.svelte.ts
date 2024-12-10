import type { UserInfo } from "../management/types.svelte";

export interface ApodInfo {
  title: string;
  date: Date;
  image: string;
  user: UserInfo;
}

export interface ApodData {
  title: string;
  user?: UserInfo;
}

export interface Apod {
  image: string;
}
