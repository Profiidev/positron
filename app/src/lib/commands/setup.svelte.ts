import { invoke } from '@tauri-apps/api/core';

export const setup = async (url: string) => {
  try {
    await invoke('setup', { url });
    return true;
  } catch {
    return false;
  }
};

export interface SetupStatus {
  url?: string;
}

export const setupStatus = async () => {
  try {
    return await invoke<SetupStatus>('setup_status');
  } catch {
    return undefined;
  }
};
