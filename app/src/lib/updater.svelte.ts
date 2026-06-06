import { goto } from '$app/navigation';
import { page } from '$app/state';
import { toast } from '@profidev/pleiades/components/util/general';
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

let connected = $state(true);
export const isConnected = () => connected;

const handleMessage = (message: UpdateMessage) => {
  switch (message.type) {
    case UpdateMessageType.TokenInvalid: {
      if (page.url.pathname !== '/auth') {
        goto('/auth').catch(() => {});
      }
      break;
    }
    case UpdateMessageType.Disconnected: {
      if (connected) {
        toast.warning('Failed to connect to server');
      }
      connected = false;
      break;
    }
    case UpdateMessageType.Connected: {
      if (!connected) {
        toast.success('Connection restored');
      }
      connected = true;
      break;
    }
    default: {
      break;
    }
  }
};
