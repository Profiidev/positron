export enum Permission {
  SETTINGS_VIEW = 'settings:view',
  SETTINGS_EDIT = 'settings:edit',
  GROUP_VIEW = 'group:view',
  GROUP_EDIT = 'group:edit',
  USER_VIEW = 'user:view',
  USER_EDIT = 'user:edit',
  OAUTH_CLIENT_VIEW = 'oauth_client:view',
  OAUTH_CLIENT_EDIT = 'oauth_client:edit',
  OAUTH_SCOPE_VIEW = 'oauth_scope:view',
  OAUTH_SCOPE_EDIT = 'oauth_scope:edit',
  OAUTH_POLICY_VIEW = 'oauth_policy:view',
  OAUTH_POLICY_EDIT = 'oauth_policy:edit',
  APOD_LIST = 'apod:list',
  APOD_SELECT = 'apod:select'
}

export const avatarUrl = '/api/user/info/avatar';

export const DEFAULT_SCOPES = ['openid', 'profile', 'email', 'image'];
