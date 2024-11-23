import { UpdateType } from "../ws/types.svelte";
import { create_updater } from "../ws/updater.svelte";
import { list_groups } from "./group.svelte";
import {
  list_clients,
  list_clients_group,
  list_clients_user,
  list_scope_names,
} from "./oauth_clients.svelte";
import { list_policies } from "./oauth_policy.svelte";
import { list_policy_info, list_scopes } from "./oauth_scope.svelte";
import type {
  Group,
  GroupInfo,
  OAuthClientInfo,
  OAuthPolicy,
  OAuthPolicyInfo,
  OAuthScope,
  User,
  UserInfo,
} from "./types.svelte";
import { list_users } from "./user.svelte";

export const user_list = create_updater<User[]>(UpdateType.User, list_users);

export const user_info_list = create_updater<UserInfo[]>(
  UpdateType.User,
  list_clients_user,
);

export const group_list = create_updater<Group[]>(
  UpdateType.Group,
  list_groups,
);

export const group_info_list = create_updater<GroupInfo[]>(
  UpdateType.Group,
  list_clients_group,
);

export const oauth_scope_list = create_updater<OAuthScope[]>(
  UpdateType.OAuthScope,
  list_scopes,
);

export const oauth_scope_names = create_updater<string[]>(
  UpdateType.OAuthScope,
  list_scope_names,
);

export const oauth_policy_list = create_updater<OAuthPolicy[]>(
  UpdateType.OAuthPolicy,
  list_policies,
);

export const oauth_policy_info_list = create_updater<OAuthPolicyInfo[]>(
  UpdateType.OAuthPolicy,
  list_policy_info,
);

export const oauth_client_list = create_updater<OAuthClientInfo[]>(
  UpdateType.OAuthClient,
  list_clients,
);
