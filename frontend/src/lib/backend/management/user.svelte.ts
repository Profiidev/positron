import { getEncrypt } from '../auth/password.svelte';
import {
  get,
  post,
  ContentType,
  ResponseType
} from 'positron-components/backend';
import type { Permission, User } from './types.svelte';

export const list_users = async () => {
  let ret = await get<User[]>('/management/user/list', ResponseType.Json);
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const user_edit = async (
  user: string,
  name: string,
  permissions: Permission[]
) => {
  return await post<undefined>(
    '/management/user/edit',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      user,
      name,
      permissions
    })
  );
};

export const create_user = async (
  name: string,
  email: string,
  password: string
) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return;
  }

  let encrypted_password = encrypt.encrypt(password);
  return await post<undefined>(
    '/management/user/create',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      name,
      email,
      password: encrypted_password
    })
  );
};

export const remove_user = async (uuid: string) => {
  return await post<undefined>(
    '/management/user/delete',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      uuid
    })
  );
};
