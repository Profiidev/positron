import { invoke } from '@tauri-apps/api/core';

export const authStatus = async () => {
  try {
    return await invoke<boolean>('auth_status');
  } catch {
    return undefined;
  }
};

export const startAuth = async () => {
  try {
    return await invoke<string>('start_auth');
  } catch {
    return undefined;
  }
};

export const confirmCode = async (code: string) => {
  try {
    await invoke('confirm_code', { code });
    return true;
  } catch {
    return undefined;
  }
};

export const logout = async () => {
  try {
    await invoke('logout');
    return true;
  } catch {
    return undefined;
  }
};
