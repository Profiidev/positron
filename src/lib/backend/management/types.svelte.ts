export interface User {
  uuid: string;
  name: string;
  image: string;
  email: string;
  last_login: string;
  permissions: Permission[];
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
