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
  import SimpleTable from 'positron-components/components/table/simple-table.svelte';
  import Multiselect from 'positron-components/components/table/multiselect.svelte';
  import { Label } from 'positron-components/components/ui/label';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import { RequestError } from 'positron-components/backend';
  import { createSchema, editSchema, deleteSchema } from './schema.svelte';
  import { columns } from './table.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import type { FormValue } from 'positron-components/components/form/types';

  let userInfo = $derived(userData.value?.[0]);
  let users = $derived(user_list.value);

  const editItemFn = async (item: User) => {
    return await user_edit(item.uuid, item.name, item.permissions);
  };

  const createItemFn = async (form: FormValue<typeof createSchema>) => {
    return await create_user(form.name, form.email, form.password);
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
  {createSchema}
  {editSchema}
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
