<script lang="ts">
  import {
    create_scope,
    delete_scope,
    edit_scope,
  } from "$lib/backend/management/oauth_scope.svelte";
  import {
    oauth_policy_info_list,
    oauth_scope_list,
  } from "$lib/backend/management/stores.svelte";
  import {
    Permission,
    type OAuthPolicyInfo,
  } from "$lib/backend/management/types.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import SimpleTable from "$lib/components/table/simple-table.svelte";
  import type { SuperValidated } from "sveltekit-superforms";
  import type { PageServerData } from "./$types";
  import { createSchema, editSchema, deleteSchema } from "./schema.svelte";
  import { columns } from "./table.svelte";
  import FormInput from "$lib/components/form/form-input.svelte";
  import { Label } from "$lib/components/ui/label";
  import Multiselect from "$lib/components/table/multiselect.svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let scopes = $derived(oauth_scope_list.value);
  let policies = $derived(oauth_policy_info_list.value);
  let policy: OAuthPolicyInfo[] = $state([]);

  const createItemFn = async (form: SuperValidated<any>) => {
    let scope = form.data;
    scope.policy = policy;
    return await create_scope(scope);
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
  data={scopes}
  filter_keys={["name", "scope", "uuid"]}
  {columns}
  label="Scope"
  {createItemFn}
  editItemFn={edit_scope}
  deleteItemFn={delete_scope}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Scopes"
  description="Modify, create, delete scopes and manage their settings here"
  createPermission={Permission.OAuthClientCreate}
  {createForm}
  {editForm}
  {deleteForm}
  errorMappings={{
    [RequestError.Conflict]: {
      field: "name",
      error: "Name already taken",
    },
  }}
>
  {#snippet editDialog({ props, item })}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput label="Scope" placeholder="Scope" key="scope" {...props} />
    <Label>Policies</Label>
    <Multiselect
      {...props}
      label="Policies"
      data={policies?.map((g) => ({
        label: g.name,
        value: g,
      })) || []}
      selected={item.policy}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/snippet}
  {#snippet createDialog({ props })}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput label="Scope" placeholder="Scope" key="scope" {...props} />
    <Label>Policies</Label>
    <Multiselect
      {...props}
      label="Policies"
      data={policies?.map((g) => ({
        label: g.name,
        value: g,
      })) || []}
      selected={policy}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/snippet}
</SimpleTable>
