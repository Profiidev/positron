import type { ComponentType } from "svelte";
import type { Icon } from "lucide-svelte";
import Nav from "./nav.svelte";

type Option = {
  title: string;
  label?: string;
  selected: boolean;
  icon: ComponentType<Icon>;
};

export {
  Nav,
  type Option
}