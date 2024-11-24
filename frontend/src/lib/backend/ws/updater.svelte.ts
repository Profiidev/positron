import { browser } from "$app/environment";
import { PUBLIC_BACKEND_URL } from "$env/static/public";
import { tick } from "svelte";
import { UpdateType } from "./types.svelte";

let updater: WebSocket | undefined | false = $state(browser && undefined);
let updater_cbs = new Map<UpdateType, Map<string, () => void>>();
let interval: number;

export const connect_updater = () => {
  if (updater === false) {
    return;
  }

  if (updater) {
    updater.close();
    clearInterval(interval);
  }

  updater = new WebSocket(`${PUBLIC_BACKEND_URL}/ws/updater`);

  updater.addEventListener("message", (event) => {
    let msg: UpdateType = JSON.parse(event.data);
    updater_cbs
      .get(msg)
      ?.values()
      .forEach((cb) => cb());
  });

  updater.addEventListener("close", () => {
    setTimeout(() => {
      connect_updater();
    }, 1000);
  });

  interval = setInterval(() => {
    if (!updater) {
      clearInterval(interval);
      return;
    }

    updater.send("heartbeat");
  }, 10000);

  updater_cbs.values().forEach((types) => types.values().forEach((cb) => cb()));
};

export const register_cb = (type: UpdateType, cb: () => void) => {
  let uuid = crypto.randomUUID().toString();

  let existing = updater_cbs.get(type) || new Map();
  existing.set(uuid, cb);
  updater_cbs.set(type, existing);

  return uuid;
};

export const unregister_cb = (uuid: string, type: UpdateType) => {
  let type_cbs = updater_cbs.get(type);
  type_cbs?.delete(uuid);
};

export const create_updater = <T>(
  type: UpdateType,
  update: () => Promise<T | undefined>,
) => {
  let value: T | undefined = $state();

  let subscribers = 0;
  let uuid: string;

  return {
    get value() {
      if ($effect.tracking()) {
        $effect(() => {
          if (subscribers === 0) {
            uuid = register_cb(type, async () => {
              value = await update();
            });
            update().then((v) => (value = v));
          }

          subscribers++;

          return () => {
            tick().then(() => {
              subscribers--;
              if (subscribers === 0) {
                unregister_cb(uuid, type);
              }
            });
          };
        });
      }

      return value;
    },
  };
};
