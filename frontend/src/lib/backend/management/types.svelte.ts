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

export enum Permission {
  //user page
  UserList = "UserList",
  UserEdit = "UserEdit",
  UserCreate = "UserCreate",
  UserDelete = "UserDelete",

  //group page
  GroupList = "GroupList",
  GroupEdit = "GroupEdit",
  GroupCreate = "GroupCreate",
  GroupDelete = "GroupDelete",
}

enum PermissionGroups {
  User = "User",
  Group = "Group",
}

export const getPermissionGroups = () => {
  return Object.keys(PermissionGroups).map((g) => {
    return {
      label: g,
      items: Object.keys(Permission)
        .filter((p) => p.startsWith(g))
        .map((p) => ({
          label: p,
          value: p as Permission,
        })),
    };
  });
};
