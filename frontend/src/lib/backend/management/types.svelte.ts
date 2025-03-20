export interface User {
  uuid: string;
  name: string;
  image: string;
  email: string;
  last_login: string;
  permissions: Permission[];
  access_level: number;
}

export interface Group {
  uuid: string;
  name: string;
  permissions: Permission[];
  access_level: number;
  users: UserInfo[];
}

export interface UserInfo {
  name: string;
  uuid: string;
}

export interface GroupInfo {
  name: string;
  uuid: string;
}

export interface OAuthClientInfo {
  name: string;
  client_id: string;
  redirect_uri: string;
  additional_redirect_uris: string[];
  default_scope: string;
  group_access: GroupInfo[];
  user_access: UserInfo[];
}

export interface OAuthClientCreate {
  secret: string;
  client_id: string;
}

export interface OAuthPolicy {
  name: string;
  uuid: string;
  claim: string;
  default: string;
  group: [GroupInfo, string][];
}

export interface OAuthPolicyInfo {
  name: string;
  uuid: string;
}

export interface OAuthPolicyCreate {
  name: string;
  claim: string;
  default: string;
  group: [GroupInfo, string][];
}

export interface OAuthScope {
  name: string;
  uuid: string;
  scope: string;
  policy: OAuthPolicyInfo[];
}

export interface OAuthScopeCreate {
  name: string;
  scope: string;
  policy: OAuthPolicyInfo[];
}

export enum Permission {
  //user page
  UserList = 'UserList',
  UserEdit = 'UserEdit',
  UserCreate = 'UserCreate',
  UserDelete = 'UserDelete',

  //group page
  GroupList = 'GroupList',
  GroupEdit = 'GroupEdit',
  GroupCreate = 'GroupCreate',
  GroupDelete = 'GroupDelete',

  //oauth client page
  OAuthClientCreate = 'OAuthClientCreate',
  OAuthClientDelete = 'OAuthClientDelete',
  OAuthClientList = 'OAuthClientList',
  OAuthClientEdit = 'OAuthClientEdit',

  //apod list
  ApodList = 'ApodList',
  ApodSelect = 'ApodSelect'
}

enum PermissionGroups {
  User = 'User',
  Group = 'Group',
  OAuthClient = 'OAuthClient',
  Apod = 'Apod'
}

export const getPermissionGroups = () => {
  return Object.keys(PermissionGroups).map((g) => {
    return {
      label: g,
      items: Object.keys(Permission)
        .filter((p) => p.startsWith(g))
        .map((p) => ({
          label: p,
          value: p as Permission
        }))
    };
  });
};
