<script lang="ts">
  import { userInfo as user_info } from "$lib/backend/account/info.svelte";
  import {
    Permission,
    type OAuthClientCreate,
    type OAuthClientInfo,
  } from "$lib/backend/management/types.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import type { Row } from "@tanstack/table-core";
  import { columns } from "./table.svelte";
  import {
    create_client,
    delete_client,
    edit_client,
    reset_client_secret,
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
  import { Separator } from "$lib/components/ui/separator";
  import { PUBLIC_BACKEND_URL } from "$env/static/public";
  import { Button } from "$lib/components/ui/button";
  import { LoaderCircle, RotateCw } from "lucide-svelte";
  import {
    group_info_list,
    oauth_client_list,
    oauth_scope_names,
    user_info_list,
  } from "$lib/backend/management/stores.svelte";

  let isLoading = $state(false);
  let clients = $derived(oauth_client_list.value);
  let groups = $derived(group_info_list.value);
  let users = $derived(user_info_list.value);
  let scope_names = $derived(oauth_scope_names.value);
  let userInfo = $derived(user_info.value);

  let client: OAuthClientInfo | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);
  let name = $state("");
  let redirect_uri = $state("");
  let additional_redirect_uri = $state("");
  let additional_redirect_uri_edit = $state("");
  let scope: string[] = $state([]);
  let startCreate: OAuthClientCreate | undefined = $state();
  let newSecret = $state("");
  let scope_edit: string[] = $state([]);

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
  let backendURLs = $derived([
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
      value: `${PUBLIC_BACKEND_URL}/oauth/logout/${startCreate?.client_id || client?.client_id || ""}`,
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
      value: `${PUBLIC_BACKEND_URL}/oauth/${startCreate?.client_id || client?.client_id || ""}/.well-known/openid-configuration`,
    },
  ]);

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
      startCreate = ret;
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

    let ret = await create_client(
      name,
      redirect_uri,
      other_uris,
      scope.join(" "),
    );
    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating client";
      }
    } else {
      toast.success("Created Client");
      name = "";
      redirect_uri = "";
      additional_redirect_uri = "";
      scope = [];
    }
  };

  const editClient = (client_id: string) => {
    client = clients?.find((client) => client.client_id === client_id);
    additional_redirect_uri_edit =
      client?.additional_redirect_uris.join(" ") || "";
    scope_edit = client?.default_scope.split(" ").filter((s) => s !== "") || [];
    newSecret = "";
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

    client.default_scope = scope_edit.join(" ");

    let ret = await edit_client(client);

    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while updating client";
      }
    } else {
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
      toast.success("Client deleted");
    }
  };

  const resetSecret = async () => {
    if (!client) {
      return;
    }

    isLoading = true;
    let ret = await reset_client_secret(client.client_id);
    isLoading = false;

    if (ret) {
      newSecret = ret.secret;
    } else {
      return "Error while creating new secret";
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
  class="md:max-w-[750px]"
>
  {#if client && userInfo}
    <div class="h-full w-full grid md:grid-cols-[1fr_60px_1fr]">
      <div class="space-y-1 grid gap-1">
        {#each backendURLs as info}
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
            onclick={resetSecret}
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
        <Input id="id" value={client.client_id} readonly />
        <Label for="name">Name</Label>
        <Input id="name" placeholder="Name" bind:value={client.name} />
        <Label>Scope</Label>
        <Multiselect
          label="Scope"
          data={scope_names?.map((s) => ({
            label: s,
            value: s,
          })) || []}
          selected={scope_edit}
        />
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
      </div>
    </div>
  {/if}
</FormDialog>
<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Clients</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete clients and manage their settings here
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
        class="md:max-w-[750px]"
      >
        <div class="h-full w-full grid md:grid-cols-[1fr_60px_1fr]">
          <div class="space-y-1">
            {#each backendURLs as info}
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
            <Label for="name">Name</Label>
            <Input
              id="name"
              placeholder="Name"
              required
              disabled={isLoading}
              bind:value={name}
            />
            <Label for="scope">Scope</Label>
            <Multiselect
              label="Scope"
              data={scope_names?.map((s) => ({
                label: s,
                value: s,
              })) || []}
              selected={scope}
            />
            <Label for="uri">Default Redirect URI</Label>
            <Input
              id="uri"
              placeholder="Redirect URI"
              required
              bind:value={redirect_uri}
            />
            <Label for="add_uri"
              >Additional Redirect URIs (space separated)</Label
            >
            <Input
              id="add_uri"
              placeholder="Additional Redirect URIs"
              bind:value={additional_redirect_uri}
            />
          </div>
        </div>
      </FormDialog>
    {/if}
  </Table>
</div>
