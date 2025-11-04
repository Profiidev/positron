<script lang="ts">
  import { user_list } from '$lib/backend/management/stores.svelte';
  import {
    getPermissionGroups,
    Permission,
    type User
  } from '$lib/backend/management/types.svelte';
  import {
    create_user,
    remove_user,
    user_edit
  } from '$lib/backend/management/user.svelte';
  import {
    SimpleTable,
    Multiselect
  } from 'positron-components/components/table';
  import { Label } from 'positron-components/components/ui';
  import {
    FormInput,
    type FormType
  } from 'positron-components/components/form';
  import { RequestError } from 'positron-components/backend';
  import type { PageServerData } from './$types';
  import { createSchema, editSchema, deleteSchema } from './schema.svelte';
  import { columns } from './table.svelte';
  import { userData } from '$lib/backend/account/info.svelte';

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let userInfo = $derived(userData.value?.[0]);
  let users = $derived(user_list.value);

  const editItemFn = async (item: User) => {
    return await user_edit(item.uuid, item.name, item.permissions);
  };

  const createItemFn = async (form: FormType<any>) => {
    return await create_user(
      form.data.name,
      form.data.email,
      form.data.password
    );
  };
</script>

<SimpleTable
  data={users}
  filter_keys={['name']}
  {columns}
  label="User"
  {createItemFn}
  {editItemFn}
  deleteItemFn={remove_user}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Users"
  description="Modify, create, delete users and manage their permissions here"
  createForm={data.createForm}
  {createSchema}
  editForm={data.editForm}
  {editSchema}
  deleteForm={data.deleteForm}
  {deleteSchema}
  errorMappings={{
    [RequestError.Conflict]: {
      field: '',
      error: ''
    }
  }}
  createButtonDisabled={!userInfo?.permissions.includes(Permission.UserCreate)}
  columnData={{
    access_level: userInfo?.access_level ?? 0,
    allowed_permissions: userInfo?.permissions ?? []
  }}
>
  {#snippet editDialog({ props, item })}
    {#if userInfo}
      <FormInput label="Name" placeholder="Name" key="name" {...props} />
      <Label for="permissions">Permissions</Label>
      <Multiselect
        {...props}
        label="Permissions"
        data={getPermissionGroups()}
        filter={(i) => userInfo.permissions.includes(i.label as Permission)}
        selected={item.permissions}
      />
    {/if}
  {/snippet}
  {#snippet createDialog({ props })}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput label="Email" placeholder="Email" key="email" {...props} />
    <FormInput
      label="Password"
      placeholder="Password"
      key="password"
      type="password"
      {...props}
    />
  {/snippet}
</SimpleTable>
