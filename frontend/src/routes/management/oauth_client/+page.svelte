<script lang="ts">
  import { getUserInfo } from "$lib/backend/account/info.svelte";
  import {
    Permission,
    type GroupInfo,
    type OAuthClientInfo,
    type UserInfo,
  } from "$lib/backend/management/types.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import type { Row } from "@tanstack/table-core";
  import { columns } from "./table.svelte";
  import {
    create_client,
    delete_client,
    edit_client,
    list_clients,
    list_clients_group,
    list_clients_user,
    start_create_client,
  } from "$lib/backend/management/oauth_clients.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import { toast } from "svelte-sonner";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import Multiselect from "$lib/components/table/multiselect.svelte";
  import Table from "$lib/components/table/table.svelte";
  import { isUrl } from "$lib/util/other.svelte";

  let isLoading = $state(false);
  let clients: OAuthClientInfo[] | undefined = $state();
  let groups: GroupInfo[] | undefined = $state();
  let users: UserInfo[] | undefined = $state();
  list_clients().then((client) => (clients = client));
  list_clients_group().then((group) => (groups = group));
  list_clients_user().then((user) => (users = user));
  let userInfo = $derived(getUserInfo());

  let client: OAuthClientInfo | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);
  let name = $state("");
  let redirect_uri = $state("");
  let additional_redirect_uri = $state("");
  let additional_redirect_uri_edit = $state("");
  let scope = $state("");

  const filterFn = (
    row: Row<OAuthClientInfo>,
    id: string,
    filterValues: any,
  ) => {
    const info = [
      row.original.name,
      row.original.client_id,
      row.original.default_scope,
    ]
      .filter(Boolean)
      .join(" ");

    let searchTerms = Array.isArray(filterValues)
      ? filterValues
      : [filterValues];
    return searchTerms.some((term) => info.includes(term.toLowerCase()));
  };

  let table = $state(
    createTable(
      [],
      columns(
        [],
        () => {},
        () => {},
      ),
      filterFn,
    ),
  );

  $effect(() => {
    table = createTable(
      clients || [],
      columns(userInfo?.permissions || [], editClient, deleteClient),
      filterFn,
    );
  });

  const startCreateClient = async () => {
    let ret = await start_create_client();

    if (ret) {
      return true;
    } else {
      toast.error("Error while starting client creation");
      return false;
    }
  };

  const createClient = async () => {
    if (!isUrl(redirect_uri)) {
      return "Set a valid redirect URI";
    }

    let other_uris = additional_redirect_uri.split(" ").filter((u) => u !== "");
    if (!other_uris.every((u) => isUrl(u))) {
      return "Set valid additional redirect URIs";
    }

    let ret = await create_client(name, redirect_uri, other_uris, scope);
    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating client";
      }
    } else {
      updateClients();
      toast.success("Created Client");
      name = "";
      redirect_uri = "";
      additional_redirect_uri = "";
      scope = "";
    }
  };

  const updateClients = async () => {
    await list_clients().then((client) => (clients = client));
  };

  const editClient = (client_id: string) => {
    client = clients?.find((client) => client.client_id === client_id);
    additional_redirect_uri_edit =
      client?.additional_redirect_uris.join(" ") || "";
    editOpen = true;
  };

  const deleteClient = (client_id: string) => {
    client = clients?.find((client) => client.client_id === client_id);
    deleteOpen = true;
  };

  const editClientConfirm = async () => {
    if (!client) {
      return;
    }

    if (!isUrl(client.redirect_uri)) {
      return "Set a valid redirect URI";
    }

    client.additional_redirect_uris = additional_redirect_uri_edit
      .split(" ")
      .filter((u) => u !== "");
    if (!client.additional_redirect_uris.every((u) => isUrl(u))) {
      return "Set valid additional redirect URIs";
    }

    let ret = await edit_client(client);

    if (ret) {
      return "Error while updating client";
    } else {
      updateClients();
      toast.success("Client updated");
    }
  };

  const deleteClientConfirm = async () => {
    if (!client) {
      return;
    }

    let ret = await delete_client(client.client_id);

    if (ret) {
      return "Error while deleting client";
    } else {
      updateClients();
      toast.success("Client deleted");
    }
  };
</script>

<FormDialog
  title="Delete Client"
  description={`Do you really want to delete the client ${client?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteClientConfirm}
  bind:open={deleteOpen}
></FormDialog>
<FormDialog
  title="Edit Client"
  description={`Edit the client info for ${client?.name} below`}
  confirm="Confirm"
  onsubmit={editClientConfirm}
  bind:open={editOpen}
>
  {#if client && userInfo}
    <Label for="name">Name</Label>
    <Input id="name" placeholder="Name" bind:value={client.name} />
    <Label for="scope">Scope</Label>
    <Input id="scope" placeholder="Scope" bind:value={client.default_scope} />
    <Label for="uri">Default Redirect URI</Label>
    <Input
      id="uri"
      placeholder="Redirect URI"
      required
      bind:value={client.redirect_uri}
    />
    <Label for="add_uri">Additional Redirect URIs (space separated)</Label>
    <Input
      id="add_uri"
      placeholder="Additional Redirect URIs"
      bind:value={additional_redirect_uri_edit}
    />
    <Label>Groups</Label>
    <Multiselect
      label="Groups"
      data={groups?.map((g) => ({
        label: g.name,
        value: g,
      })) || []}
      selected={client.group_access}
      display={(u) => u.name}
      compare={(a, b) => a.uuid === b.uuid}
    />
    <Label>Users</Label>
    <Multiselect
      label="Users"
      data={users?.map((u) => ({
        label: u.name,
        value: u,
      })) || []}
      selected={client.user_access}
      display={(u) => u.name}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/if}
</FormDialog>
<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Clients</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete clients and manage settings here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if userInfo?.permissions.includes(Permission.OAuthClientCreate)}
      <FormDialog
        title="Create Client"
        description="Enter the details for the new client below"
        confirm="Create"
        trigger={{
          text: "Create Client",
          variant: "secondary",
          class: "ml-2",
          loadIcon: true,
        }}
        onsubmit={createClient}
        onopen={startCreateClient}
      >
        <Label for="name">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="scope">Scope</Label>
        <Input
          id="scope"
          placeholder="Scope"
          required
          disabled={isLoading}
          bind:value={scope}
        />
        <Label for="uri">Default Redirect URI</Label>
        <Input
          id="uri"
          placeholder="Redirect URI"
          required
          bind:value={redirect_uri}
        />
        <Label for="add_uri">Additional Redirect URIs (space separated)</Label>
        <Input
          id="add_uri"
          placeholder="Additional Redirect URIs"
          bind:value={additional_redirect_uri}
        />
      </FormDialog>
    {/if}
  </Table>
</div>
