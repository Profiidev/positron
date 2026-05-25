<script lang="ts">
  import FormCheckbox from '@profidev/pleiades/components/form/form-checkbox.svelte';
  import { Permission } from '$lib/permissions.svelte';
  import type {
    FormPath,
    FormValue,
    SuperForm
  } from '@profidev/pleiades/components/form/types';
  import type { groupSettings } from './schema.svelte';
  import type { UserInfo } from '$lib/client';
  import { title } from 'valibot';

  interface Props {
    user?: UserInfo;
    readonly: boolean;
    disabled: boolean;
    formData: SuperForm<FormValue<typeof groupSettings>>;
  }

  const { user, readonly, ...props }: Props = $props();
</script>

<h5>Permissions</h5>
<div class="ml-4">
  {@render permission({
    header: 'Settings',
    read: Permission.SETTINGS_VIEW,
    read_key: 'settings$view',
    read_label: 'View Settings',
    write: Permission.SETTINGS_EDIT,
    write_key: 'settings$edit',
    write_label: 'Edit Settings'
  })}
  {@render permission({
    header: 'Groups',
    read: Permission.GROUP_VIEW,
    read_key: 'group$view',
    read_label: 'View Groups',
    write: Permission.GROUP_EDIT,
    write_key: 'group$edit',
    write_label: 'Edit Groups'
  })}
  {@render permission({
    header: 'Users',
    read: Permission.USER_VIEW,
    read_key: 'user$view',
    read_label: 'View Users',
    write: Permission.USER_EDIT,
    write_key: 'user$edit',
    write_label: 'Edit Users'
  })}
  {@render permission({
    header: 'OAuth / Oidc Client',
    read: Permission.OAUTH_CLIENT_VIEW,
    read_key: 'oauth_client$view',
    read_label: 'View Clients',
    write: Permission.OAUTH_CLIENT_EDIT,
    write_key: 'oauth_client$edit',
    write_label: 'Edit Clients'
  })}
  {@render permission({
    header: 'OAuth / Oidc Scope',
    read: Permission.OAUTH_SCOPE_VIEW,
    read_label: 'View Scopes',
    read_key: 'oauth_scope$view',
    write: Permission.OAUTH_SCOPE_EDIT,
    write_key: 'oauth_scope$edit',
    write_label: 'Edit Scopes'
  })}
  {@render permission({
    header: 'OAuth / Oidc Policy',
    read: Permission.OAUTH_POLICY_VIEW,
    read_key: 'oauth_policy$view',
    read_label: 'View Policies',
    write: Permission.OAUTH_POLICY_EDIT,
    write_key: 'oauth_policy$edit',
    write_label: 'Edit Policies'
  })}
</div>

{#snippet permission({
  header,
  read,
  read_key,
  read_label,
  write,
  write_key,
  write_label
}: {
  header: string;
  read: Permission;
  read_key: FormPath<FormValue<typeof groupSettings>>;
  read_label: string;
  write: Permission;
  write_key: FormPath<FormValue<typeof groupSettings>>;
  write_label: string;
})}
  {#if user?.permissions.includes(read) || user?.permissions.includes(write)}
    <h6>{header}</h6>
    <div class="ml-4">
      {#if user?.permissions.includes(read)}
        <FormCheckbox
          {...props}
          key={read_key}
          label={read_label}
          disabled={readonly}
          {readonly}
        />
      {/if}
      {#if user?.permissions.includes(write)}
        <FormCheckbox
          {...props}
          key={write_key}
          label={write_label}
          disabled={readonly}
          {readonly}
        />
      {/if}
    </div>
  {/if}
{/snippet}
