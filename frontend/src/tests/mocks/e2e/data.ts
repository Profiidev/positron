import { Permission } from '$lib/permissions.svelte';

/** Scenario name read from the `mock_scenario` cookie (set by the e2e tests). */
export type Scenario = 'default' | 'empty';

export const scenarioOf = (cookies: Record<string, string>): Scenario =>
  cookies.mock_scenario === 'empty' ? 'empty' : 'default';

/** Admin user with every permission, so admin pages render full controls. */
export const adminUser = {
  email: 'admin@example.com',
  name: 'Ada Admin',
  permissions: Object.values(Permission),
  totp_enabled: false,
  uuid: 'user-admin'
};

const simpleUser = { id: 'user-1', name: 'Bob User' };
const simpleGroup = { name: 'Admins', uuid: 'group-admins' };
const simpleScope = { name: 'profile', scope: 'profile', uuid: 'scope-1' };
const simplePolicy = { name: 'Group Policy', uuid: 'policy-1' };

export const isSetup = {
  db_backend: 'sqlite',
  is_setup: true,
  storage_backend: 'local'
};

export const authConfig = { mail_enabled: true };
export const accountSettings = { o_auth_instant_confirm: false };
export const mailActive = { active: true };
export const mailSettings = {
  from_env: [] as string[],
  settings: {
    smtp_enabled: false,
    smtp_from_name: 'Positron',
    smtp_use_tls: false
  }
};
export const siteUrl = { url: 'https://positron.example' };
export const oidcSettings = {
  client_id: '',
  issuer: '',
  scopes: [] as string[]
};

export const groups = {
  default: {
    admin_group: 'group-admins',
    groups: [
      {
        id: 'group-admins',
        name: 'Admins',
        permissions: [Permission.USER_VIEW, Permission.GROUP_VIEW],
        users: [simpleUser]
      },
      { id: 'group-staff', name: 'Staff', permissions: [], users: [] }
    ]
  },
  empty: { admin_group: undefined, groups: [] as unknown[] }
};

export const users = {
  default: [
    {
      email: 'bob@example.com',
      groups: [simpleGroup],
      name: 'Bob User',
      uuid: 'user-1'
    },
    {
      email: 'cara@example.com',
      groups: [],
      name: 'Cara User',
      uuid: 'user-2'
    }
  ],
  empty: [] as unknown[]
};

export const notes = {
  default: [
    {
      id: 'note-1',
      is_owner: true,
      owner: simpleUser,
      preview: 'First note preview',
      shared_with: [],
      title: 'My First Note'
    }
  ],
  empty: [] as unknown[]
};

export const oauthClients = {
  default: [
    {
      additional_redirect_uris: [],
      client_id: 'client-1',
      confidential: true,
      default_scope: [simpleScope],
      group_access: [simpleGroup],
      name: 'Dashboard App',
      redirect_uri: 'https://app.example/callback',
      require_pkce: false,
      user_access: [simpleUser]
    }
  ],
  empty: [] as unknown[]
};

export const oauthScopes = {
  default: [
    {
      name: 'profile',
      policies: [simplePolicy],
      scope: 'profile',
      uuid: 'scope-1'
    }
  ],
  empty: [] as unknown[]
};

export const oauthPolicies = {
  default: [
    {
      claim: 'groups',
      content: [
        {
          content: 'admin',
          group_id: 'group-admins',
          group_name: 'Admins',
          index: 0
        }
      ],
      default: 'user',
      name: 'Group Policy',
      uuid: 'policy-1'
    }
  ],
  empty: [] as unknown[]
};

export const apodList = {
  default: [
    { date: '2024-01-02', title: 'Spiral Galaxy', user: simpleUser },
    { date: '2024-01-01', title: 'Nebula', user: simpleUser }
  ],
  empty: [] as unknown[]
};

export const apodImageInfo = {
  date: '2024-01-02',
  title: 'Spiral Galaxy',
  user: simpleUser
};

export const passkeys = {
  default: [
    {
      created: '2024-01-01T00:00:00Z',
      name: 'YubiKey',
      used: '2024-01-02T00:00:00Z'
    }
  ],
  empty: [] as unknown[]
};

// Detail payloads.
export const groupDetails = {
  admin_group: 'group-admins',
  group: {
    id: 'group-admins',
    name: 'Admins',
    permissions: [Permission.USER_VIEW],
    users: [simpleUser]
  }
};

export const userDetails = {
  email: 'bob@example.com',
  groups: [simpleGroup],
  name: 'Bob User',
  oidc_user: false,
  permissions: [Permission.USER_VIEW],
  uuid: 'user-1'
};

export const oauthClientDetails = oauthClients.default[0];
export const oauthScopeDetails = oauthScopes.default[0];
export const oauthPolicyDetails = oauthPolicies.default[0];
export const noteDetails = {
  id: 'note-1',
  is_owner: true,
  owner: simpleUser,
  shared_with: [],
  title: 'My First Note'
};

export const simpleUsers = { default: [simpleUser], empty: [] as unknown[] };
export const simpleGroups = { default: [simpleGroup], empty: [] as unknown[] };
export const simpleScopes = { default: [simpleScope], empty: [] as unknown[] };
export const simplePolicies = {
  default: [simplePolicy],
  empty: [] as unknown[]
};
