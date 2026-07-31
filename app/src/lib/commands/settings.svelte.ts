import { invoke } from '@tauri-apps/api/core';

export enum HorizontalLayout {
  Left = 'left',
  Center = 'center',
  Right = 'right'
}

export enum VerticalLayout {
  Top = 'top',
  Center = 'center',
  Bottom = 'bottom'
}

export interface Settings {
  horizontalLayout: HorizontalLayout;
  verticalLayout: VerticalLayout;
}

export const getSettings = async () => {
  try {
    return await invoke<Settings>('get_settings');
  } catch {
    return undefined;
  }
};

export const saveSettings = async (settings: Settings) => {
  try {
    await invoke('save_settings', { settings });
    return true;
  } catch {
    return false;
  }
};
