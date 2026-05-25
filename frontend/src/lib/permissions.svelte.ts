export enum Permission {
  SETTINGS_VIEW = 'settings:view',
  SETTINGS_EDIT = 'settings:edit',
  GROUP_VIEW = 'group:view',
  GROUP_EDIT = 'group:edit',
  USER_VIEW = 'user:view',
  USER_EDIT = 'user:edit',
  AOUTH_CLIENT_VIEW = 'oauth_client:view',
  OAUTH_CLIENT_EDIT = 'oauth_client:edit'
}

export const avatarUrl = '/api/user/info/avatar';
