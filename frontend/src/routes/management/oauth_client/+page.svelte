<script lang="ts">
  import { PUBLIC_BACKEND_URL } from '$env/static/public';
  import {
    create_client,
    delete_client,
    edit_client,
    reset_client_secret,
    start_create_client
  } from '$lib/backend/management/oauth_clients.svelte';
  import {
    group_info_list,
    oauth_client_list,
    oauth_scope_names,
    user_info_list
  } from '$lib/backend/management/stores.svelte';
  import {
    Permission,
    type OAuthClientCreate,
    type OAuthClientInfo
  } from '$lib/backend/management/types.svelte';
  import {
    Multiselect,
    SimpleTable
  } from 'positron-components/components/table';
  import {
    Label,
    Input,
    Separator,
    Button,
    toast
  } from 'positron-components/components/ui';
  import {
    FormInput,
    FormSwitch,
    type FormType
  } from 'positron-components/components/form';
  import { RequestError } from 'positron-components/backend';
  import { deepCopy } from 'positron-components/util';
  import { columns } from './table.svelte';
  import { createSchema, editSchema, deleteSchema } from './schema.svelte';
  import { RotateCw } from '@lucide/svelte';
  import type { PageServerData } from './$types';
  import { userData } from '$lib/backend/account/info.svelte';
  import HidableInput from '$lib/components/form/HidableInput.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let clients = $derived(oauth_client_list.value);
  let groups = $derived(group_info_list.value);
  let users = $derived(user_info_list.value);
  let scope_names = $derived(oauth_scope_names.value);
  let userInfo = $derived(userData.value?.[0]);

  let isLoading = $state(false);
  let scope: string[] = $state([]);
  let startCreate: OAuthClientCreate | undefined = $state();
  let newSecret = $state('');

  let backendURLs = (client_id: string) => [
    {
      name: 'Authorization URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/authorize`
    },
    {
      name: 'Token URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/token`
    },
    {
      name: 'Userinfo URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/user`
    },
    {
      name: 'Logout URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/logout/${client_id}`
    },
    {
      name: 'Revoke URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/revoke`
    },
    {
      name: 'JWKs URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/jwks`
    },
    {
      name: 'OIDC Configuration URL',
      value: `${PUBLIC_BACKEND_URL}/oauth/.well-known/openid-configuration`
    }
  ];

  const createItemFn = async (form: FormType<any>) => {
    return await create_client(
      form.data.name,
      form.data.redirect_uri,
      form.data.additional_redirect_uris,
      scope.join(' '),
      form.data.confidential
    );
  };

  const resetSecret = async (item: OAuthClientInfo) => {
    isLoading = true;
    let ret = await reset_client_secret(item.client_id);
    isLoading = false;

    if (ret) {
      newSecret = ret.secret;
    } else {
      return 'Error while creating new secret';
    }
  };

  const startCreateClient = async () => {
    scope = [];
    let ret = await start_create_client();

    if (ret) {
      startCreate = ret;
      return true;
    } else {
      toast.error('Error while starting client creation');
      return false;
    }
  };

  const editClient = async (client: OAuthClientInfo) => {
    let clientLocal = deepCopy(client);
    clientLocal.default_scope = scope.join(' ');
    return await edit_client(clientLocal);
  };

  const createForm = {
    schema: createSchema,
    form: data.createForm
  };

  const editForm = {
    schema: editSchema,
    form: data.editForm
  };

  const deleteForm = {
    schema: deleteSchema,
    form: data.deleteForm
  };
</script>

<SimpleTable
  data={clients}
  filter_keys={['name', 'client_id', 'default_scope']}
  {columns}
  label="Client"
  {createItemFn}
  editItemFn={editClient}
  deleteItemFn={delete_client}
  toId={(item) => item.client_id}
  display={(item) => item?.name}
  title="Clients"
  description="Modify, create, delete clients and manage their settings here"
  {createForm}
  {editForm}
  {deleteForm}
  errorMappings={{
    [RequestError.Conflict]: {
      field: 'name',
      error: 'Name already taken'
    }
  }}
  startCreate={startCreateClient}
  startEdit={(item) => {
    scope = item.default_scope.split(' ');
    newSecret = '';
  }}
  createClass="md:max-w-[750px]"
  editClass="md:max-w-[750px]"
  createButtonDisabled={!userInfo?.permissions.includes(
    Permission.OAuthClientCreate
  )}
  columnData={userInfo?.permissions ?? []}
>
  {#snippet editDialog({ props, item })}
    <div class="grid h-full w-full md:grid-cols-[1fr_60px_1fr]">
      <div class="grid gap-1 space-y-1">
        {#each backendURLs(item.client_id) as info}
          <Label for={info.name}>{info.name}</Label>
          <Input id={info.name} value={info.value} readonly />
        {/each}
        <Label for="secret" class="flex"
          >Client Secret
          {#if newSecret !== ''}
            <span class="text-destructive ml-auto"
              >WILL NOT BE VISIBLE AGAIN</span
            >
          {/if}
        </Label>
        {#if !item.confidential}
          <p class="ml-2">Client is not confidential</p>
        {:else if newSecret === ''}
          <Button
            disabled={isLoading}
            variant="destructive"
            onclick={() => resetSecret(item)}
          >
            <RotateCw class="mr-2 h-4 w-4 {isLoading ? 'animate-spin' : ''}" />
            Reset</Button
          >
        {:else}
          <Input id="secret" value={newSecret} readonly />
        {/if}
      </div>
      <Separator orientation="vertical" class="mx-[30px]" />
      <div class="grid h-fit gap-1 space-y-1">
        <Label for="id">Client ID</Label>
        <Input id="id" value={item.client_id} readonly />
        <FormInput label="Name" placeholder="Name" key="name" {...props} />
        <Label>Scope</Label>
        <Multiselect
          {...props}
          label="Scope"
          data={scope_names?.map((s) => ({
            label: s,
            value: s
          })) || []}
          bind:selected={scope}
        />
        <FormInput
          label="Default Redirect URI"
          placeholder="URI"
          key="redirect_uri"
          {...props}
        />
        <FormInput
          label="Additional Redirect URIs (space separated)"
          placeholder="URIs"
          key="additional_redirect_uris"
          {...props}
        />
        <Label>Groups</Label>
        <Multiselect
          {...props}
          label="Groups"
          data={groups?.map((g) => ({
            label: g.name,
            value: g
          })) || []}
          bind:selected={item.group_access}
          compare={(a, b) => a.uuid === b.uuid}
        />
        <Label>Users</Label>
        <Multiselect
          {...props}
          label="Users"
          data={users?.map((u) => ({
            label: u.name,
            value: u
          })) || []}
          bind:selected={item.user_access}
          compare={(a, b) => a.uuid === b.uuid}
        />
      </div>
    </div>
  {/snippet}
  {#snippet createDialog({ props })}
    <div class="grid h-full w-full md:grid-cols-[1fr_60px_1fr]">
      <div class="grid gap-1 space-y-1">
        {#each backendURLs(startCreate?.client_id || '') as info}
          <Label for={info.name}>{info.name}</Label>
          <Input id={info.name} value={info.value} readonly />
        {/each}
      </div>
      <Separator orientation="vertical" class="mx-[30px]" />
      <div class="grid h-fit gap-1 space-y-1">
        <Label for="client-id" class="flex">Client ID</Label>
        <Input id="client-id" value={startCreate?.client_id} readonly />
        <FormSwitch label="Confidential" key="confidential" {...props} />
        <HidableInput
          id="client-secret"
          value={startCreate?.secret}
          key="confidential"
          {...props}
        >
          Client Secret <span class="text-destructive ml-auto"
            >WILL NOT BE VISIBLE AGAIN</span
          >
        </HidableInput>
        <FormInput label="Name" placeholder="Name" key="name" {...props} />
        <Label for="scope">Scope</Label>
        <Multiselect
          {...props}
          label="Scope"
          data={scope_names?.map((s) => ({
            label: s,
            value: s
          })) || []}
          bind:selected={scope}
        />
        <FormInput
          label="Default Redirect URI"
          placeholder="URI"
          key="redirect_uri"
          {...props}
        />
        <FormInput
          label="Additional Redirect URIs (space separated)"
          placeholder="URIs"
          key="additional_redirect_uris"
          {...props}
        />
      </div>
    </div>
  {/snippet}
</SimpleTable>
