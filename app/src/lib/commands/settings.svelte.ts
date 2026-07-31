import { invoke } from '@tauri-apps/api/core';

export enum HorizontalLayout {
  Left = 'Left',
  Center = 'Center',
  Right = 'Right'
}

export enum VerticalLayout {
  Top = 'Top',
  Center = 'Center',
  Bottom = 'Bottom'
}

export interface Settings {
  horizontal_layout: HorizontalLayout;
  vertical_layout: VerticalLayout;
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
