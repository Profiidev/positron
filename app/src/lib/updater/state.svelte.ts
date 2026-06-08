import { tick } from 'svelte';
import { UpdateMessageType } from './types.svelte';
import { setupStatus } from '$lib/commands/setup.svelte';
import { authStatus } from '$lib/commands/auth.svelte';

const updater_cbs = new Map<UpdateMessageType, Map<string, () => void>>();

export const triggerUpdates = (type?: UpdateMessageType) => {
  for (const [type_key, type_cbs] of updater_cbs) {
    if (type === undefined || type_key === type) {
      for (const [_, cb] of type_cbs) {
        cb();
      }
    }
  }
};

const register_cb = (type: UpdateMessageType, cb: () => void) => {
  const uuid = crypto.randomUUID() as string;

  const existing = updater_cbs.get(type) || new Map();
  existing.set(uuid, cb);
  updater_cbs.set(type, existing);

  return uuid;
};

const unregister_cb = (uuid: string, type: UpdateMessageType) => {
  const type_cbs = updater_cbs.get(type);
  type_cbs?.delete(uuid);
};

const create_updater = <T>(
  type: UpdateMessageType,
  update: () => Promise<T | undefined>
) => {
  // oxlint-disable-next-line no-null
  let value: T | undefined | null = $state(null);

  let subscribers = 0;
  let uuid = '';

  const runUpdate = async () => update().then((v) => (value = v));

  return {
    update: async () => {
      await runUpdate();
    },
    get value() {
      if ($effect.tracking()) {
        $effect(() => {
          if (subscribers === 0) {
            uuid = register_cb(type, () => {
              runUpdate().catch(() => {});
            });
            runUpdate().catch(() => {});
          }

          subscribers += 1;

          return () => {
            tick()
              .then(() => {
                subscribers -= 1;
                if (subscribers === 0) {
                  unregister_cb(uuid, type);
                }
              })
              .catch(() => {});
          };
        });
      }

      return value;
    }
  };
};

export const setupStatusState = create_updater(
  UpdateMessageType.SetupUpdated,
  setupStatus
);

export const authStatusState = create_updater(
  UpdateMessageType.AuthStatusUpdated,
  authStatus
);
