<script lang="ts">
  import {
    create_group,
    delete_group,
    edit_group,
  } from "$lib/backend/management/group.svelte";
  import {
    group_list,
    user_info_list,
  } from "$lib/backend/management/stores.svelte";
  import {
    getPermissionGroups,
    Permission,
  } from "$lib/backend/management/types.svelte";
  import Multiselect from "$lib/components/table/multiselect.svelte";
  import SimpleTable from "$lib/components/table/simple-table.svelte";
  import { Label } from "$lib/components/ui/label";
  import { columns } from "./table.svelte";
  import { createSchema, deleteSchema, editSchema } from "./schema.svelte";
  import type { PageServerData } from "./$types";
  import FormInput from "$lib/components/form/form-input.svelte";
  import { userData } from "$lib/backend/account/info.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import type { SuperValidated } from "sveltekit-superforms";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let groups = $derived(group_list.value);
  let users = $derived(user_info_list.value);
  let userInfo = $derived(userData.value?.[0]);

  const createItemFn = async (form: SuperValidated<any>) => {
    return await create_group(form.data.name, Number(form.data.access_level));
  };

  const createForm = {
    schema: createSchema,
    form: data.createForm,
  };

  const editForm = {
    schema: editSchema,
    form: data.editForm,
  };

  const deleteForm = {
    schema: deleteSchema,
    form: data.deleteForm,
  };
</script>

<SimpleTable
  data={groups}
  filter_keys={["name", "uuid"]}
  {columns}
  label="Group"
  {createItemFn}
  editItemFn={edit_group}
  deleteItemFn={delete_group}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Groups"
  description="Modify, create, delete groups and manage their permissions and members here"
  createPermission={Permission.GroupCreate}
  {createForm}
  {editForm}
  {deleteForm}
  errorMappings={{
    [RequestError.Conflict]: {
      field: "name",
      error: "Name already taken",
    },
    [RequestError.Unauthorized]: {
      field: "access_level",
      error: "You can only use access levels below yours",
    },
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
        value: u,
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
