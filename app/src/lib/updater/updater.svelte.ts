import { goto } from '$app/navigation';
import { page } from '$app/state';
import { toast } from '@profidev/pleiades/components/util/general';
import { Channel, invoke } from '@tauri-apps/api/core';
import { triggerUpdates } from './state.svelte';
import { type UpdateMessage, UpdateMessageType } from './types.svelte';

export const setOnline = async (online: boolean): Promise<boolean> => {
  try {
    await invoke('set_online', { online });
    return true;
  } catch {
    return false;
  }
};

export const startListener = async () => {
  const channel = new Channel<UpdateMessage>();
  // oxlint-disable-next-line prefer-add-event-listener
  channel.onmessage = handleMessage;

  const uuid = await invoke<string>('connect_updater', {
    channel
  });

  triggerUpdates();

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
      if (page.route.id !== '/auth') {
        goto('/auth').catch(() => {});
      }
      break;
    }
    case UpdateMessageType.Disconnected: {
      connected = false;
      break;
    }
    case UpdateMessageType.Connected: {
      connected = true;
      triggerUpdates();
      break;
    }
    case UpdateMessageType.CodeExchangeFailed: {
      toast.error('Failed to exchange code');
      goto('/auth').catch(() => {});
      break;
    }
    case UpdateMessageType.CodeExchangeMissingCode: {
      toast.error('Code exchange missing code');
      goto('/auth').catch(() => {});
      break;
    }
    case UpdateMessageType.CodeExchangeMissingVerifier: {
      toast.error('Code exchange missing verifier');
      goto('/auth').catch(() => {});
      break;
    }
    case UpdateMessageType.AuthSuccess: {
      toast.success('Authenticated successfully');
      goto('/').catch(() => {});
      break;
    }
    case UpdateMessageType.ConfirmAuth: {
      const params = new URLSearchParams({ code: message.code });
      if (message.redirect) {
        params.set('redirect', message.redirect);
      }
      goto(`/login?${params.toString()}`).catch(() => {});
      break;
    }
    case UpdateMessageType.ConfirmAuthMissingCode: {
      toast.error('Code missing for confirmation');
      goto('/').catch(() => {});
      break;
    }
    default: {
      break;
    }
  }

  triggerUpdates(message.type);
};
