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
  OAuthScope = 'OAuthScope'
}

export type UpdateMessage =
  | {
      type:
        | UpdateType.User
        | UpdateType.Group
        | UpdateType.OAuthClient
        | UpdateType.OAuthScope;
      uuid: string;
    }
  | {
      type: UpdateType.Settings | UpdateType.UserPermissions;
    };

export const connectWebsocket = (user: string) => connect(user, handleMessage);
export const disconnectWebsocket = () => disconnect();

const handleMessage = (msg: UpdateMessage, user: string) => {
  switch (msg.type) {
    case UpdateType.Settings: {
      invalidate((url) => url.pathname.startsWith('/api/settings')).catch(() => {});
      break;
    }
    case UpdateType.User: {
      invalidate('/api/user/management').catch(() => {});
      invalidate(`/api/user/management/${msg.uuid}`).catch(() => {});
      invalidate('/api/group/users').catch(() => {});
      invalidate('/api/oauth_management/client/users').catch(() => {});
      invalidate(`/api/user/info/avatar/${msg.uuid}`).catch(() => {});
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
    default: {
      break;
    }
  }
};
