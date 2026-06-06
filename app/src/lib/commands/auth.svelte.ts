import { invoke } from '@tauri-apps/api/core';

export const authStatus = async () => {
  try {
    return await invoke<boolean>('auth_status');
  } catch {
    return undefined;
  }
};
