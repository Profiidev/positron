import Settings from '@lucide/svelte/icons/settings';
import House from '@lucide/svelte/icons/house';
import { Permission } from '$lib/permissions.svelte';
import Users from '@lucide/svelte/icons/users';
import User from '@lucide/svelte/icons/user';
import KeyRound from '@lucide/svelte/icons/key-round';
import Goal from '@lucide/svelte/icons/goal';
import UserKey from '@lucide/svelte/icons/user-key';
import Telescope from '@lucide/svelte/icons/telescope';
import NotepadText from '@lucide/svelte/icons/notepad-text';
import type { NavGroup } from '@profidev/pleiades/components/nav/sidebar/types';

export const items: NavGroup[] = [
  {
    items: [{ href: '/', icon: House, label: 'Overview' }],
    label: 'Overview'
  },
  {
    items: [
      {
        href: '/notes',
        icon: NotepadText,
        label: 'Notes'
      },
      {
        href: '/apod',
        icon: Telescope,
        label: 'APOD',
        requiredPermission: Permission.APOD_LIST
      }
    ],
    label: 'Services'
  },
  {
    items: [
      {
        href: '/oauth-client',
        icon: KeyRound,
        label: 'Clients',
        requiredPermission: Permission.OAUTH_CLIENT_VIEW
      },
      {
        href: '/oauth-scope',
        icon: Goal,
        label: 'Scopes',
        requiredPermission: Permission.OAUTH_SCOPE_VIEW
      },
      {
        href: '/oauth-policy',
        icon: UserKey,
        label: 'Policies',
        requiredPermission: Permission.OAUTH_POLICY_VIEW
      }
    ],
    label: 'OAuth / Oidc'
  },
  {
    items: [
      {
        href: '/users',
        icon: User,
        label: 'Users',
        requiredPermission: Permission.USER_VIEW
      },
      {
        href: '/groups',
        icon: Users,
        label: 'Groups',
        requiredPermission: Permission.GROUP_VIEW
      },
      {
        href: '/settings',
        icon: Settings,
        label: 'Settings',
        requiredPermission: Permission.SETTINGS_VIEW
      }
    ],
    label: 'Administration'
  }
];

export const noAuthPaths = [
  '/login',
  '/setup',
  '/password',
  '/password/forgot',
  '/password/reset',
  '/notes/share/[id]'
];

export const noSidebarPaths = [
  ...noAuthPaths,
  '/oauth',
  '/oauth/logout',
  '/auth/app'
];
