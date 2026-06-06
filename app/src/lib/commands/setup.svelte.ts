import { invoke } from '@tauri-apps/api/core';

export const setup = async (url: string) => {
  try {
    await invoke('setup', { payload: { url } });
    return true;
  } catch {
    return false;
  }
};

export interface SetupStatus {
  url_set: boolean;
}

export const setupStatus = async () => {
  try {
    const result = await invoke('setup_status');
    // oxlint-disable-next-line no-unsafe-type-assertion
    return result as SetupStatus;
  } catch {
    return undefined;
  }
};
