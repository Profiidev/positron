import { tick, untrack } from "svelte";

const createKeyedWatcher = () => {
  let watchers = new Map();

  return {
    watch(setup: () => () => void) {
      if ($effect.tracking()) {
        $effect(() => {
          let entry = watchers.get(setup);
          if (!entry) {
            const cleanup = untrack(setup);
            entry = [0, cleanup];
            watchers.set(setup, entry);
          }
          entry[0]++;

          return () => {
            tick().then(() => {
              entry[0]--;
              if (entry[0] === 0) {
                entry[1]?.();
                watchers.delete(setup);
              }
            });
          };
        });
      }
    },
  };
};

export const interval = <T>(update: () => T, timeout: number) => {
  let value = $state(update());

  const setup = () => {
    const id = setInterval(() => {
      value = update();
    }, timeout);
    return () => clearInterval(id);
  };

  const watcher = createKeyedWatcher();

  return {
    get value() {
      watcher.watch(setup);
      return value;
    },
  };
};

export const wait_for = (condition: () => boolean, intervalTime = 100) => {
  return new Promise((resolve) => {
    const interval = setInterval(() => {
      if (condition()) {
        clearInterval(interval);
        resolve(undefined);
      }
    }, intervalTime);
  });
};
