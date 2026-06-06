import { goto } from '$app/navigation';
import { page } from '$app/state';
import { Channel, invoke } from '@tauri-apps/api/core';

export enum UpdateMessageType {
  TokenInvalid = 'TokenInvalid',
  Disconnected = 'Disconnected',
  Connected = 'Connected'
}

export type UpdateMessage = {
  type:
    | UpdateMessageType.Disconnected
    | UpdateMessageType.TokenInvalid
    | UpdateMessageType.Connected;
};

export const startListener = async () => {
  const channel = new Channel<UpdateMessage>();
  // oxlint-disable-next-line prefer-add-event-listener
  channel.onmessage = handleMessage;

  const uuid = await invoke<string>('connect_updater', {
    channel
  });

  return () => {
    invoke('disconnect_updater', {
      uuid
    }).catch(() => {});
  };
};

const handleMessage = (message: UpdateMessage) => {
  switch (message.type) {
    case UpdateMessageType.TokenInvalid: {
      if (page.url.pathname !== '/auth') {
        goto('/auth').catch(() => {});
      }
      break;
    }
    case UpdateMessageType.Disconnected: {
      break;
    }
    case UpdateMessageType.Connected: {
      break;
    }
    default: {
      break;
    }
  }
};
