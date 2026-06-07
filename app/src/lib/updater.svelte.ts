import { goto } from '$app/navigation';
import { page } from '$app/state';
import { toast } from '@profidev/pleiades/components/util/general';
import { Channel, invoke } from '@tauri-apps/api/core';

export enum UpdateMessageType {
  TokenInvalid = 'TokenInvalid',
  Disconnected = 'Disconnected',
  Connected = 'Connected',
  CodeExchangeFailed = 'CodeExchangeFailed',
  CodeExchangeMissingCode = 'CodeExchangeMissingCode',
  CodeExchangeMissingVerifier = 'CodeExchangeMissingVerifier',
  AuthSuccess = 'AuthSuccess',
  ConfirmAuth = 'ConfirmAuth',
  ConfirmAuthMissingCode = 'ConfirmAuthMissingCode'
}

// oxlint-disable-next-line consistent-type-definitions
export type UpdateMessage =
  | {
      type:
        | UpdateMessageType.Disconnected
        | UpdateMessageType.TokenInvalid
        | UpdateMessageType.Connected
        | UpdateMessageType.CodeExchangeFailed
        | UpdateMessageType.CodeExchangeMissingCode
        | UpdateMessageType.CodeExchangeMissingVerifier
        | UpdateMessageType.AuthSuccess
        | UpdateMessageType.ConfirmAuthMissingCode;
    }
  | {
      type: UpdateMessageType.ConfirmAuth;
      code: string;
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
      goto(`/login?code=${message.code}`).catch(() => {});
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
};
