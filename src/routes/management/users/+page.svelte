<script lang="ts">
  import {
    getPermissions,
    getPriority,
    updatePermissions,
  } from "$lib/backend/account/info.svelte";
  import { Permission, type User } from "$lib/backend/management/types.svelte";
  import {
    create,
    list,
    update_permissions,
  } from "$lib/backend/management/user.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import Table from "$lib/components/table/table.svelte";
  import { toast } from "svelte-sonner";
  import { columns } from "./table.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { fetch_key } from "$lib/backend/auth/password.svelte";
  import type { Row } from "@tanstack/table-core";

  const updateUsers = async () => {
    await list().then((user) => (users = user));
  };

  const filterFn = (row: Row<User>, id: string, filterValues: any) => {
    const info = [row.original.email, row.original.name, row.original.uuid]
      .filter(Boolean)
      .join(" ");

    let searchTerms = Array.isArray(filterValues)
      ? filterValues
      : [filterValues];
    return searchTerms.some((term) => info.includes(term.toLowerCase()));
  };

  let users: User[] | undefined = $state();
  list().then((user) => (users = user));
  let table = $state(
    createTable(
      [],
      columns([], Number.MAX_SAFE_INTEGER, updateUsers),
      filterFn,
    ),
  );
  let allowed_permissions = $derived(getPermissions());
  let priority = $derived(getPriority());
  let name = $state("");
  let email = $state("");
  let password = $state("");
  let isLoading = $state(false);

  $effect(() => {
    table = createTable(
      users || [],
      columns(
        allowed_permissions || [],
        priority ?? Number.MAX_SAFE_INTEGER,
        updateUsers,
        permissionSelect,
      ),
      filterFn,
    );
  });

  const permissionSelect = (user: string, value: Permission, add: boolean) => {
    update_permissions(user, value, add).then((ret) => {
      if (ret !== null) {
        toast.error("Error while updating");
      }
    });
  };

  const createUser = async () => {
    let ret = await create(name, email, password);
    if (ret !== null) {
      await fetch_key();
      return "Error while creating user";
    } else {
      await list().then((user) => (users = user));
      toast.success("Created User");
    }
  };
</script>

<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Users</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete users and manage their permissions here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if allowed_permissions?.includes(Permission.UserCreate)}
      <FormDialog
        title="Create User"
        description="Enter the details for the new user below"
        confirm="Create"
        trigger={{
          text: "Create User",
          variant: "secondary",
          class: "ml-2",
        }}
        onsubmit={createUser}
      >
        <Label for="name" class="sr-only">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="email" class="sr-only">Email</Label>
        <Input
          id="email"
          placeholder="Email"
          required
          disabled={isLoading}
          bind:value={email}
        />
        <Label for="password" class="sr-only">Password</Label>
        <Input
          id="passowrd"
          placeholder="Password"
          required
          disabled={isLoading}
          bind:value={password}
        />
      </FormDialog>
    {/if}
  </Table>
</div>
