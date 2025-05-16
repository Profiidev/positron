import { browser } from '$app/environment';
import { PUBLIC_BACKEND_URL, PUBLIC_IS_APP } from '$env/static/public';
import { tick } from 'svelte';
import { UpdateType } from './types.svelte';
import { sleep } from 'positron-components/util';
import type TauriWebSocket from '@tauri-apps/plugin-websocket';
import { BASE_URL } from '../util.svelte';

type CondWebsocket<T extends string> = T extends 'true'
  ? TauriWebSocket
  : WebSocket;

let updater: CondWebsocket<typeof PUBLIC_IS_APP> | undefined | false = $state(
  browser && undefined
);
let updater_cbs = new Map<UpdateType, Map<string, () => void>>();
let interval: number;

export const connect_updater = () => {
  if (updater === false) {
    return;
  }

  if (updater) {
    return;
  }

  create_websocket();
};

const create_websocket =
  PUBLIC_IS_APP !== 'true'
    ? () => {
        updater = new WebSocket(`${PUBLIC_BACKEND_URL}/ws/updater`);

        updater.addEventListener('message', (event) => {
          let msg: UpdateType = JSON.parse(event.data);
          Array.from(updater_cbs.get(msg)?.values() || []).forEach((cb) =>
            cb()
          );
        });

        updater.addEventListener('close', async () => {
          clearInterval(interval);
          await sleep(1000);
          create_websocket();
        });

        interval = setInterval(() => {
          if (
            !updater ||
            updater.readyState === updater.CLOSING ||
            updater.readyState === updater.CLOSED
          ) {
            clearInterval(interval);
            return;
          }

          updater.send('heartbeat');
        }, 10000);

        Array.from(updater_cbs.values()).forEach((types) =>
          Array.from(types.values()).forEach((cb) => cb())
        );
      }
    : async () => {
        let token: string | undefined = undefined;
        try {
          token = await (
            await import('@tauri-apps/api/core')
          ).invoke('get_cookie', {
            key: 'token',
            url: BASE_URL
          });
        } catch (_) {}
        console.log('token', token);

        const WebSocket = (await import('@tauri-apps/plugin-websocket'))
          .default;
        try {
          // @ts-ignore
          updater = await WebSocket.connect(
            `${PUBLIC_BACKEND_URL.replace('https://', 'wss://').replace('http://', 'ws://')}/ws/updater`,
            {
              headers: {
                Cookie: `token=${token}`
              }
            }
          );
        } catch (e) {
          console.error('Failed to connect to updater', e);
          clearInterval(interval);
          await sleep(1000);
          create_websocket();
          return;
        }

        // @ts-ignore
        updater.addListener(async (event) => {
          switch (event.type) {
            case 'Text':
              let msg: UpdateType = JSON.parse(event.data);
              Array.from(updater_cbs.get(msg)?.values() || []).forEach((cb) =>
                cb()
              );
              break;
            case 'Close':
              clearInterval(interval);
              await sleep(1000);
              create_websocket();
              break;
          }
        });

        interval = setInterval(() => {
          if (!updater) {
            clearInterval(interval);
            return;
          }

          updater.send('heartbeat');
        }, 10000);

        Array.from(updater_cbs.values()).forEach((types) =>
          Array.from(types.values()).forEach((cb) => cb())
        );
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
  update: () => Promise<T | undefined>
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
    update: async () => {
      update().then((v) => (value = v));
    }
  };
};
