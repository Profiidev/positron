import { invoke } from '@tauri-apps/api/core';

export interface UserInfo {
  uuid: string;
  name: string;
  email: string;
}

export const userInfo = async () => {
  try {
    return await invoke<UserInfo>('user_info');
  } catch {
    return undefined;
  }
};

export const userAvatar = async () => {
  try {
    const res = await invoke<number[]>('user_avatar');
    const uint8Array = new Uint8Array(res);
    const blob = new Blob([uint8Array], { type: 'image/webp' });
    return URL.createObjectURL(blob);
  } catch {
    return undefined;
  }
};

export const anyUserAvatar = async (uuid: string) => {
  try {
    const res = await invoke<number[]>('any_user_avatar', { uuid });
    const uint8Array = new Uint8Array(res);
    const blob = new Blob([uint8Array], { type: 'image/webp' });
    return URL.createObjectURL(blob);
  } catch {
    return undefined;
  }
};
