<script lang="ts">
  import {
    create_group,
    delete_group,
    edit_group
  } from '$lib/backend/management/group.svelte';
  import {
    group_list,
    user_info_list
  } from '$lib/backend/management/stores.svelte';
  import {
    getPermissionGroups,
    Permission
  } from '$lib/backend/management/types.svelte';
  import SimpleTable from 'positron-components/components/table/simple-table.svelte';
  import Multiselect from 'positron-components/components/table/multiselect.svelte';
  import { Label } from 'positron-components/components/ui/label';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import { RequestError } from 'positron-components/backend';
  import { columns } from './table.svelte';
  import { createSchema, deleteSchema, editSchema } from './schema.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import type { FormValue } from 'positron-components/components/form/types';

  let groups = $derived(group_list.value);
  let users = $derived(user_info_list.value);
  let userInfo = $derived(userData.value?.[0]);

  const createItemFn = async (form: FormValue<typeof createSchema>) => {
    return await create_group(form.name, Number(form.access_level));
  };
</script>

<SimpleTable
  data={groups}
  filter_keys={['name', 'uuid']}
  {columns}
  label="Group"
  {createItemFn}
  editItemFn={edit_group}
  deleteItemFn={delete_group}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Groups"
  description="Modify, create, delete groups and manage their permissions and members here"
  {createSchema}
  {editSchema}
  {deleteSchema}
  errorMappings={{
    [RequestError.Conflict]: {
      field: 'name',
      error: 'Name already taken'
    },
    [RequestError.Unauthorized]: {
      field: 'access_level',
      error: 'You can only use access levels below yours'
    }
  }}
  createButtonDisabled={!userInfo?.permissions.includes(Permission.GroupCreate)}
  columnData={{
    access_level: userInfo?.access_level ?? 0,
    allowed_permissions: userInfo?.permissions ?? []
  }}
>
  {#snippet editDialog({ props, item })}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput
      label="Access Level"
      placeholder="Access Level"
      key="access_level"
      type="number"
      {...props}
    />
    <Label>Permissions</Label>
    <Multiselect
      {...props}
      label="Permissions"
      data={getPermissionGroups()}
      filter={(i) => userInfo!.permissions.includes(i.label as Permission)}
      bind:selected={item.permissions}
    />
    <Label>Users</Label>
    <Multiselect
      {...props}
      label="Users"
      data={users?.map((u) => ({
        label: u.name,
        value: u
      })) || []}
      bind:selected={item.users}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/snippet}
  {#snippet createDialog({ props })}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput
      label="Access Level"
      placeholder="Access Level"
      key="access_level"
      type="number"
      {...props}
    />
  {/snippet}
</SimpleTable>
