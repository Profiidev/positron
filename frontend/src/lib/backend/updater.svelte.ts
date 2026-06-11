import { invalidate } from '$app/navigation';
import {
  connectWebsocket as connect,
  disconnectWebsocket as disconnect
} from '@profidev/pleiades/backend';

export enum UpdateType {
  Settings = 'Settings',
  User = 'User',
  UserPermissions = 'UserPermissions',
  Group = 'Group',
  OAuthClient = 'OAuthClient',
  OAuthScope = 'OAuthScope',
  OAuthPolicy = 'OAuthPolicy',
  Passkey = 'Passkey',
  Apod = 'Apod',
  Note = 'Note'
}

export type UpdateMessage =
  | {
      type:
        | UpdateType.User
        | UpdateType.Group
        | UpdateType.OAuthClient
        | UpdateType.OAuthScope
        | UpdateType.OAuthPolicy
        | UpdateType.Note;
      uuid: string;
    }
  | {
      type:
        | UpdateType.Settings
        | UpdateType.UserPermissions
        | UpdateType.Passkey
        | UpdateType.Apod;
    };

export const connectWebsocket = (user: string) => connect(user, handleMessage);
export const disconnectWebsocket = () => disconnect();

const handleMessage = (msg: UpdateMessage, user: string) => {
  switch (msg.type) {
    case UpdateType.Settings: {
      invalidate((url) => url.pathname.startsWith('/api/settings')).catch(
        () => {}
      );
      break;
    }
    case UpdateType.User: {
      invalidate('/api/user/management').catch(() => {});
      invalidate(`/api/user/management/${msg.uuid}`).catch(() => {});
      invalidate('/api/group/users').catch(() => {});
      invalidate('/api/oauth_management/client/users').catch(() => {});
      invalidate(`/api/user/info/avatar/${msg.uuid}`).catch(() => {});
      invalidate(`/api/notes/management/users`).catch(() => {});
      // Same as current user
      if (msg.uuid === user) {
        invalidate('/api/user/info').catch(() => {});
      }
      break;
    }
    case UpdateType.UserPermissions: {
      invalidate('/api/user/info').catch(() => {});
      break;
    }
    case UpdateType.Group: {
      invalidate('/api/group').catch(() => {});
      invalidate(`/api/group/${msg.uuid}`).catch(() => {});
      invalidate('/api/user/management/groups').catch(() => {});
      invalidate('/api/oauth_management/client/groups').catch(() => {});
      invalidate(`/api/oauth_management/policy/groups`).catch(() => {});
      break;
    }
    case UpdateType.OAuthClient: {
      invalidate('/api/oauth_management/client').catch(() => {});
      invalidate(`/api/oauth_management/client/${msg.uuid}`).catch(() => {});
    }
    case UpdateType.OAuthScope: {
      invalidate('/api/oauth_management/scope').catch(() => {});
      invalidate(`/api/oauth_management/scope/${msg.uuid}`).catch(() => {});
      invalidate('/api/oauth_management/client/scopes').catch(() => {});
    }
    case UpdateType.OAuthPolicy: {
      invalidate('/api/oauth_management/policy').catch(() => {});
      invalidate(`/api/oauth_management/policy/${msg.uuid}`).catch(() => {});
      invalidate(`/api/oauth_management/scope/policies`).catch(() => {});
      break;
    }
    case UpdateType.Passkey: {
      invalidate('/api/auth/passkey/list').catch(() => {});
    }
    case UpdateType.Apod: {
      invalidate('/api/services/apod').catch(() => {});
      invalidate('/api/services/apod/get_image_info').catch(() => {});
      break;
    }
    case UpdateType.Note: {
      invalidate('/api/notes/management').catch(() => {});
      invalidate(`/api/notes/management/${msg.uuid}`).catch(() => {});
      break;
    }
    default: {
      break;
    }
  }
};
