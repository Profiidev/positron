<script lang="ts">
  import { user_list } from "$lib/backend/management/stores.svelte";
  import {
    getPermissionGroups,
    Permission,
    type User,
  } from "$lib/backend/management/types.svelte";
  import {
    create_user,
    remove_user,
    user_edit,
  } from "$lib/backend/management/user.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import SimpleTable from "$lib/components/table/simple-table.svelte";
  import type { SuperValidated } from "sveltekit-superforms";
  import type { PageServerData } from "./$types";
  import { createSchema, editSchema, deleteSchema } from "./schema.svelte";
  import { columns } from "./table.svelte";
  import FormInput from "$lib/components/form/form-input.svelte";
  import { Label } from "$lib/components/ui/label";
  import Multiselect from "$lib/components/table/multiselect.svelte";
  import { userData } from "$lib/backend/account/info.svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let userInfo = $derived(userData.value?.[0]);
  let users = $derived(user_list.value);

  const editItemFn = async (item: User) => {
    return await user_edit(item.uuid, item.name, item.permissions);
  };

  const createItemFn = async (form: SuperValidated<any>) => {
    return await create_user(
      form.data.name,
      form.data.email,
      form.data.password,
    );
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
  data={users}
  filter_keys={["name"]}
  {columns}
  label="User"
  {createItemFn}
  {editItemFn}
  deleteItemFn={remove_user}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Users"
  description="Modify, create, delete users and manage their permissions here"
  createPermission={Permission.UserCreate}
  {createForm}
  {editForm}
  {deleteForm}
  errorMappings={{
    [RequestError.Conflict]: {
      field: "",
      error: "",
    },
  }}
>
  {#snippet editDialog({ props, item })}
    {#if userInfo}
      <FormInput label="Name" placeholder="Name" key="name" {...props} />
      <Label for="permissions">Permissions</Label>
      <Multiselect
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
