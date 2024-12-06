<script lang="ts">
  import { PUBLIC_BACKEND_URL } from "$env/static/public";
  import {
    create_client,
    delete_client,
    edit_client,
    reset_client_secret,
    start_create_client,
  } from "$lib/backend/management/oauth_clients.svelte";
  import {
    group_info_list,
    oauth_client_list,
    oauth_scope_names,
    user_info_list,
  } from "$lib/backend/management/stores.svelte";
  import {
    Permission,
    type OAuthClientCreate,
    type OAuthClientInfo,
  } from "$lib/backend/management/types.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import Multiselect from "$lib/components/table/multiselect.svelte";
  import SimpleTable from "$lib/components/table/simple-table.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { columns } from "./table.svelte";
  import { createSchema, editSchema, deleteSchema } from "./schema.svelte";
  import FormInput from "$lib/components/form/form-input.svelte";
  import { Button } from "$lib/components/ui/button";
  import { LoaderCircle, RotateCw } from "lucide-svelte";
  import type { PageServerData } from "./$types";
  import type { SuperValidated } from "sveltekit-superforms";
  import { toast } from "svelte-sonner";
  import { deepCopy } from "$lib/util/other.svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

  let clients = $derived(oauth_client_list.value);
  let groups = $derived(group_info_list.value);
  let users = $derived(user_info_list.value);
  let scope_names = $derived(oauth_scope_names.value);

  let isLoading = $state(false);
  let scope: string[] = $state([]);
  let startCreate: OAuthClientCreate | undefined = $state();
  let newSecret = $state("");

  let clientInfo = $derived([
    {
      name: "Client ID",
      value: startCreate?.client_id,
    },
    {
      name: "Client Secret <span class='text-destructive ml-14'>WILL NOT BE VISIBLE AGAIN</span>",
      value: startCreate?.secret,
    },
  ]);
  let backendURLs = (client_id: string | undefined) => [
    {
      name: "Authorization URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/authorize`,
    },
    {
      name: "Token URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/token`,
    },
    {
      name: "Userinfo URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/user`,
    },
    {
      name: "Logout URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/logout/${startCreate?.client_id || client_id || ""}`,
    },
    {
      name: "Revoke URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/revoke`,
    },
    {
      name: "JWKs URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/jwks`,
    },
    {
      name: "OIDC Configuration URL",
      value: `${PUBLIC_BACKEND_URL}/oauth/${startCreate?.client_id || client_id || ""}/.well-known/openid-configuration`,
    },
  ];

  const createItemFn = async (form: SuperValidated<any>) => {
    return await create_client(
      form.data.name,
      form.data.redirect_uri,
      form.data.additional_redirect_uris,
      scope.join(" "),
    );
  };

  const resetSecret = async (item: OAuthClientInfo) => {
    isLoading = true;
    let ret = await reset_client_secret(item.client_id);
    isLoading = false;

    if (ret) {
      newSecret = ret.secret;
    } else {
      return "Error while creating new secret";
    }
  };

  const startCreateClient = async () => {
    scope = [];
    let ret = await start_create_client();

    if (ret) {
      startCreate = ret;
      return true;
    } else {
      toast.error("Error while starting client creation");
      return false;
    }
  };

  const editClient = async (client: OAuthClientInfo) => {
    let clientLocal = deepCopy(client);
    clientLocal.default_scope = scope.join(" ");
    return await edit_client(clientLocal);
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
  data={clients}
  filter_keys={["name", "client_id", "default_scope"]}
  {columns}
  label="Client"
  {createItemFn}
  editItemFn={editClient}
  deleteItemFn={delete_client}
  toId={(item) => item.client_id}
  display={(item) => item?.name}
  title="Clients"
  description="Modify, create, delete clients and manage their settings here"
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
  startCreate={startCreateClient}
  startEdit={(item) => {
    scope = item.default_scope.split(" ");
  }}
  createClass="md:max-w-[750px]"
  editClass="md:max-w-[750px]"
>
  {#snippet editDialog({ props, item })}
    <div class="h-full w-full grid md:grid-cols-[1fr_60px_1fr]">
      <div class="space-y-1 grid gap-1">
        {#each backendURLs(item.client_id) as info}
          <Label for={info.name}>{info.name}</Label>
          <Input id={info.name} value={info.value} readonly />
        {/each}
        <Label for="secret"
          >Client Secret
          {#if newSecret !== ""}
            <span class="text-destructive ml-14">WILL NOT BE VISIBLE AGAIN</span
            >
          {/if}
        </Label>
        {#if newSecret === ""}
          <Button
            disabled={isLoading}
            variant="destructive"
            onclick={() => resetSecret(item)}
          >
            {#if isLoading}
              <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
            {/if}
            <RotateCw class="mr-2 h-4 w-4" />
            Reset</Button
          >
        {:else}
          <Input id="secret" value={newSecret} readonly />
        {/if}
      </div>
      <Separator orientation="vertical" class="mx-[30px]" />
      <div class="h-fit space-y-1 grid gap-1">
        <Label for="id">Client ID</Label>
        <Input id="id" value={item.client_id} readonly />
        <FormInput label="Name" placeholder="Name" key="name" {...props} />
        <Label>Scope</Label>
        <Multiselect
          label="Scope"
          data={scope_names?.map((s) => ({
            label: s,
            value: s,
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
          label="Groups"
          data={groups?.map((g) => ({
            label: g.name,
            value: g,
          })) || []}
          bind:selected={item.group_access}
          compare={(a, b) => a.uuid === b.uuid}
        />
        <Label>Users</Label>
        <Multiselect
          label="Users"
          data={users?.map((u) => ({
            label: u.name,
            value: u,
          })) || []}
          bind:selected={item.user_access}
          compare={(a, b) => a.uuid === b.uuid}
        />
      </div>
    </div>
  {/snippet}
  {#snippet createDialog({ props })}
    <div class="h-full w-full grid md:grid-cols-[1fr_60px_1fr]">
      <div class="space-y-1">
        {#each backendURLs(startCreate?.client_id) as info}
          <Label for={info.name}>{info.name}</Label>
          <Input id={info.name} value={info.value} readonly />
        {/each}
      </div>
      <Separator orientation="vertical" class="mx-[30px]" />
      <div class="h-fit grid space-y-1 gap-1">
        {#each clientInfo as info}
          <Label for={info.name}>{@html info.name}</Label>
          <Input id={info.name} value={info.value} readonly />
        {/each}
        <FormInput label="Name" placeholder="Name" key="name" {...props} />
        <Label for="scope">Scope</Label>
        <Multiselect
          label="Scope"
          data={scope_names?.map((s) => ({
            label: s,
            value: s,
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
