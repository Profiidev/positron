<script lang="ts">
  import { getUserInfo } from "$lib/backend/account/info.svelte";
  import {
    Permission,
    type OAuthPolicyInfo,
    type OAuthScope,
  } from "$lib/backend/management/types.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import type { Row } from "@tanstack/table-core";
  import { columns } from "./table.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import { toast } from "svelte-sonner";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import Table from "$lib/components/table/table.svelte";
  import {
    create_scope,
    delete_scope,
    edit_scope,
    list_policy_info,
    list_scopes,
  } from "$lib/backend/management/oauth_scope.svelte";
  import Multiselect from "$lib/components/table/multiselect.svelte";

  let isLoading = $state(false);
  let scopes: OAuthScope[] | undefined = $state();
  let policies: OAuthPolicyInfo[] | undefined = $state();
  list_scopes().then((scope) => (scopes = scope));
  list_policy_info().then((policy) => (policies = policy));
  let userInfo = $derived(getUserInfo());

  let scope: OAuthScope | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);
  let name = $state("");
  let scope_name = $state("");
  let policy = $state([]);

  const filterFn = (row: Row<OAuthScope>, id: string, filterValues: any) => {
    const info = [row.original.name, row.original.scope, row.original.uuid]
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
      scopes || [],
      columns(userInfo?.permissions || [], editScope, deleteScope),
      filterFn,
    );
  });

  const createScope = async () => {
    let ret = await create_scope({
      name,
      scope: scope_name,
      policy,
    });
    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating scope";
      }
    } else {
      updateScopes();
      toast.success("Created Scope");
      name = "";
      scope_name = "";
      policy = [];
    }
  };

  const updateScopes = async () => {
    await list_scopes().then((scope) => (scopes = scope));
  };

  const editScope = (uuid: string) => {
    scope = scopes?.find((scope) => scope.uuid === uuid);
    editOpen = true;
  };

  const deleteScope = (uuid: string) => {
    scope = scopes?.find((scope) => scope.uuid === uuid);
    deleteOpen = true;
  };

  const editScopeConfirm = async () => {
    if (!scope) {
      return;
    }

    let ret = await edit_scope(scope);

    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while updating scope";
      }
    } else {
      updateScopes();
      toast.success("Scope updated");
    }
  };

  const deleteScopeConfirm = async () => {
    if (!scope) {
      return;
    }

    let ret = await delete_scope(scope.uuid);

    if (ret) {
      return "Error while deleting scope";
    } else {
      updateScopes();
      toast.success("Scope deleted");
    }
  };
</script>

<FormDialog
  title="Delete Scope"
  description={`Do you really want to delete the scope ${scope?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteScopeConfirm}
  bind:open={deleteOpen}
></FormDialog>
<FormDialog
  title="Edit Scope"
  description={`Edit the scope info for ${scope?.name} below`}
  confirm="Confirm"
  onsubmit={editScopeConfirm}
  bind:open={editOpen}
>
  {#if scope && userInfo}
    <Label for="name">Name</Label>
    <Input id="name" placeholder="Name" bind:value={scope.name} />
    <Label for="scope">Scope</Label>
    <Input id="scope" placeholder="Scope" bind:value={scope.scope} />
    <Label>Policies</Label>
    <Multiselect
      label="Policies"
      data={policies?.map((g) => ({
        label: g.name,
        value: g,
      })) || []}
      selected={scope.policy}
      display={(u) => u.name}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/if}
</FormDialog>
<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Scopes</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete scopes and manage their settings here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if userInfo?.permissions.includes(Permission.OAuthClientCreate)}
      <FormDialog
        title="Create Scope"
        description="Enter the details for the new scope below"
        confirm="Create"
        trigger={{
          text: "Create Scope",
          variant: "secondary",
          class: "ml-2",
          loadIcon: true,
        }}
        onsubmit={createScope}
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
          bind:value={scope_name}
        />
        <Label>Policies</Label>
        <Multiselect
          label="Policies"
          data={policies?.map((g) => ({
            label: g.name,
            value: g,
          })) || []}
          selected={policy}
          display={(u) => u.name}
          compare={(a, b) => a.uuid === b.uuid}
        />
      </FormDialog>
    {/if}
  </Table>
</div>
