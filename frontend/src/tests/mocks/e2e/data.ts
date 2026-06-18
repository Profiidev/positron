import { Permission } from '$lib/permissions.svelte';

/** Scenario name read from the `mock_scenario` cookie (set by the e2e tests). */
export type Scenario =
  | 'default'
  | 'empty'
  | 'readonly'
  | 'at-limit'
  | 'transfer-at-limit';

/**
 * List data only varies between `default` and `empty`; the `readonly` scenario
 * reuses the default lists and only changes the note *detail* payload (see
 * `isReadonlyNote`), so a viewer can be exercised without new list fixtures.
 */
export const scenarioOf = (
  cookies: Record<string, string>
): 'default' | 'empty' =>
  cookies.mock_scenario === 'empty' ? 'empty' : 'default';

export const notesScenarioOf = (
  cookies: Record<string, string>
): 'default' | 'empty' | 'at-limit' => {
  if (cookies.mock_scenario === 'empty') {
    return 'empty';
  }
  if (cookies.mock_scenario === 'at-limit') {
    return 'at-limit';
  }
  return 'default';
};

/** True when the note detail should be served as a view-only (can_edit) note. */
export const isReadonlyNote = (cookies: Record<string, string>): boolean =>
  cookies.mock_scenario === 'readonly';

/**
 * The public-share page treats a user whose `info` falls back to the unknown
 * email as an anonymous visitor (and keeps them on the page); any other user is
 * redirected to the authenticated note view. The `mock_anon` cookie makes the
 * `info` endpoint report no session so the anonymous branch renders.
 */
export const isAnonymous = (cookies: Record<string, string>): boolean =>
  cookies.mock_anon === '1';

/** `mock_public=edit` serves an editable public note; otherwise view-only. */
export const publicNoteOf = (cookies: Record<string, string>) =>
  cookies.mock_public === 'edit'
    ? notePublicEditDetails
    : notePublicViewDetails;

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

/** Returned when the `mock_setup=pending` cookie is set, so /setup renders. */
export const isSetupPending = {
  db_backend: 'sqlite',
  is_setup: false,
  storage_backend: 'local'
};

export const isSetupOf = (cookies: Record<string, string>) =>
  cookies.mock_setup === 'pending' ? isSetupPending : isSetup;

export const authConfig = { mail_enabled: true };
export const accountSettings = { o_auth_instant_confirm: false };
export const mailActive = { active: true };
/** `mock_mail=off` cookie disables mail, unlocking the admin-managed user
 * password/email/avatar controls that hide when mail is configured. */
export const mailActiveOf = (cookies: Record<string, string>) =>
  cookies.mock_mail === 'off' ? { active: false } : mailActive;
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

export const notesConfig = {
  'at-limit': { max_per_user: 1 },
  default: { max_per_user: 20 },
  empty: { max_per_user: 20 }
};

export const notes = {
  'at-limit': [
    {
      can_edit: true,
      id: 'note-1',
      is_owner: true,
      owner: simpleUser,
      preview: 'First note preview',
      shared_with: [],
      title: 'My First Note'
    }
  ],
  default: [
    {
      can_edit: true,
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

// A non-admin group, so the detail page renders the editable permissions
// Matrix (the admin group hides it).
export const groupStaffDetails = {
  admin_group: 'group-admins',
  group: {
    id: 'group-staff',
    name: 'Staff',
    permissions: [] as Permission[],
    users: [] as unknown[]
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

// oxlint-disable-next-line prefer-destructuring
export const oauthClientDetails = oauthClients.default[0];
// oxlint-disable-next-line prefer-destructuring
export const oauthScopeDetails = oauthScopes.default[0];
// oxlint-disable-next-line prefer-destructuring
export const oauthPolicyDetails = oauthPolicies.default[0];
export const noteDetails = {
  can_edit: true,
  id: 'note-1',
  is_owner: true,
  owner: simpleUser,
  shared_with: [],
  title: 'My First Note'
};

// A note owned by someone else and shared with the current user as view-only.
// Both is_owner (share/title/delete locked) and can_edit (editor locked) false.
export const noteDetailsReadonly = {
  can_edit: false,
  id: 'note-1',
  is_owner: false,
  owner: { id: 'user-2', name: 'Cara User' },
  shared_with: [{ ...simpleUser, access: 'view' as const }],
  title: 'My First Note'
};

// Public-share (`NoteInfoPublic`) payloads served by `infoNoteShare`. They
// Carry no `is_owner`/`shared_with` — a public visitor only sees the title,
// Owner and whether the share grants edit access.
export const notePublicViewDetails = {
  can_edit: false,
  id: 'note-1',
  owner: { id: 'user-2', name: 'Cara User' },
  title: 'Shared Public Note'
};

export const notePublicEditDetails = {
  ...notePublicViewDetails,
  can_edit: true
};

export const simpleUsers = { default: [simpleUser], empty: [] as unknown[] };
// The note owner (simpleUser) plus another user, so the note's share control
// Has someone to share with (the owner is filtered out of the options).
export const noteUsers = {
  default: [simpleUser, { id: 'user-2', name: 'Cara User' }],
  empty: [] as unknown[]
};
export const simpleGroups = { default: [simpleGroup], empty: [] as unknown[] };
export const simpleScopes = { default: [simpleScope], empty: [] as unknown[] };
export const simplePolicies = {
  default: [simplePolicy],
  empty: [] as unknown[]
};
