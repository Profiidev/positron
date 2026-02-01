import { getEncrypt } from '../auth/password.svelte';
import { ResponseType, get, post } from 'positron-components/backend';
import type { Permission, User } from './types.svelte';

export const list_users = async () => {
  let ret = await get<User[]>('/backend/management/user/list', {
    res_type: ResponseType.Json
  });
  if (Array.isArray(ret)) {
    return ret;
  }
};

export const user_edit = async (
  user: string,
  name: string,
  permissions: Permission[]
) => {
  return await post('/backend/management/user/edit', {
    body: {
      user,
      name,
      permissions
    }
  });
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
  return await post('/backend/management/user/create', {
    body: {
      name,
      email,
      password: encrypted_password
    }
  });
};

export const remove_user = async (uuid: string) => {
  return await post('/backend/management/user/delete', {
    body: {
      uuid
    }
  });
};
