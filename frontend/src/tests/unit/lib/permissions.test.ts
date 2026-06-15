import { describe, expect, it } from 'vitest';
import {
  DEFAULT_SCOPES,
  Permission,
  apodImageUrl,
  avatarUrl
} from '$lib/permissions.svelte';

describe('Permission enum', () => {
  it('maps every member to a `resource:action` string', () => {
    for (const value of Object.values(Permission)) {
      expect(value).toMatch(/^[a-z_]+:[a-z_]+$/);
    }
  });

  it('has unique values', () => {
    const values = Object.values(Permission);
    expect(new Set(values).size).toBe(values.length);
  });

  it('exposes the expected members', () => {
    expect(Permission.SETTINGS_VIEW).toBe('settings:view');
    expect(Permission.SETTINGS_EDIT).toBe('settings:edit');
    expect(Permission.GROUP_VIEW).toBe('group:view');
    expect(Permission.GROUP_EDIT).toBe('group:edit');
    expect(Permission.USER_VIEW).toBe('user:view');
    expect(Permission.USER_EDIT).toBe('user:edit');
    expect(Permission.OAUTH_CLIENT_VIEW).toBe('oauth_client:view');
    expect(Permission.OAUTH_CLIENT_EDIT).toBe('oauth_client:edit');
    expect(Permission.OAUTH_SCOPE_VIEW).toBe('oauth_scope:view');
    expect(Permission.OAUTH_SCOPE_EDIT).toBe('oauth_scope:edit');
    expect(Permission.OAUTH_POLICY_VIEW).toBe('oauth_policy:view');
    expect(Permission.OAUTH_POLICY_EDIT).toBe('oauth_policy:edit');
    expect(Permission.APOD_LIST).toBe('apod:list');
    expect(Permission.APOD_SELECT).toBe('apod:select');
  });

  it('pairs a view and edit permission for each editable resource', () => {
    const resources = new Set(
      Object.values(Permission).map((p) => p.split(':')[0])
    );
    for (const resource of resources) {
      const actions = Object.values(Permission)
        .filter((p) => p.startsWith(`${resource}:`))
        .map((p) => p.split(':')[1]);
      // Apod is list/select; everything else is view/edit.
      if (resource === 'apod') {
        expect(actions).toEqual(expect.arrayContaining(['list', 'select']));
      } else {
        expect(actions).toEqual(expect.arrayContaining(['view', 'edit']));
      }
    }
  });
});

describe('url + scope constants', () => {
  it('exposes absolute api paths', () => {
    expect(avatarUrl).toBe('/api/user/info/avatar');
    expect(apodImageUrl).toBe('/api/services/apod/get_image');
    expect(avatarUrl.startsWith('/api/')).toBe(true);
    expect(apodImageUrl.startsWith('/api/')).toBe(true);
  });

  it('defines the default OIDC scopes without duplicates', () => {
    expect(DEFAULT_SCOPES).toEqual(['openid', 'profile', 'email', 'image']);
    expect(new Set(DEFAULT_SCOPES).size).toBe(DEFAULT_SCOPES.length);
    expect(DEFAULT_SCOPES).toContain('openid');
  });
});
