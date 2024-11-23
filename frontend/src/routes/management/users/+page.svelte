<script lang="ts">
  import {
    getPermissionGroups,
    Permission,
    type User,
  } from "$lib/backend/management/types.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import Table from "$lib/components/table/table.svelte";
  import { toast } from "svelte-sonner";
  import { columns } from "./table.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { fetch_key } from "$lib/backend/auth/password.svelte";
  import type { Row } from "@tanstack/table-core";
  import {
    create_user,
    remove_user,
    user_edit,
  } from "$lib/backend/management/user.svelte";
  import { userData } from "$lib/backend/account/info.svelte";
  import Multiselect from "$lib/components/table/multiselect.svelte";
  import { user_list } from "$lib/backend/management/stores.svelte";

  const filterFn = (row: Row<User>, id: string, filterValues: any) => {
    const info = [row.original.email, row.original.name, row.original.uuid]
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
        0,
        () => {},
        () => {},
      ),
      filterFn,
    ),
  );
  let userInfo = $derived(userData.value?.[0]);
  let name = $state("");
  let email = $state("");
  let password = $state("");
  let isLoading = $state(false);
  let user: User | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);

  let users = $derived(user_list.value);

  $effect(() => {
    table = createTable(
      users || [],
      columns(
        userInfo?.permissions || [],
        userInfo?.access_level ?? 0,
        editUser,
        deleteUser,
      ),
      filterFn,
    );
  });

  const createUser = async () => {
    let ret = await create_user(name, email, password);
    if (ret) {
      await fetch_key();
      return "Error while creating user";
    } else {
      toast.success("Created User");
      email = "";
      name = "";
      password = "";
    }
  };

  const editUser = (uuid: string) => {
    user = users?.find((user) => user.uuid === uuid);
    editOpen = true;
  };

  const deleteUser = (uuid: string) => {
    user = users?.find((user) => user.uuid === uuid);
    deleteOpen = true;
  };

  const editUserConfirm = async () => {
    if (!user) {
      return;
    }

    let ret = await user_edit(user.uuid, user.name, user.permissions);

    if (ret) {
      return "Error while updating user";
    } else {
      toast.success("User updated");
    }
  };

  const deleteUserConfirm = async () => {
    if (!user) {
      return;
    }

    let ret = await remove_user(user.uuid);

    if (ret) {
      return "Error while deleting user";
    } else {
      toast.success("User deleted");
    }
  };
</script>

<FormDialog
  title="Delete User"
  description={`Do you really want to delete the user ${user?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteUserConfirm}
  bind:open={deleteOpen}
></FormDialog>
<FormDialog
  title="Edit User"
  description={`Edit the user info for ${user?.name} below`}
  confirm="Confirm"
  onsubmit={editUserConfirm}
  bind:open={editOpen}
>
  {#if user && userInfo}
    <Label for="name">Name</Label>
    <Input id="name" placeholder="name" bind:value={user.name} />
    <Label for="permissions">Permissions</Label>
    <Multiselect
      label="Permissions"
      data={getPermissionGroups()}
      filter={(i) => userInfo.permissions.includes(i.label as Permission)}
      selected={user.permissions}
    />
  {/if}
</FormDialog>
<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Users</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete users and manage their permissions here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if userInfo?.permissions.includes(Permission.UserCreate)}
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
        <Label for="name">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="email">Email</Label>
        <Input
          id="email"
          placeholder="Email"
          type="email"
          required
          disabled={isLoading}
          bind:value={email}
        />
        <Label for="password">Password</Label>
        <Input
          id="passowrd"
          placeholder="Password"
          type="password"
          required
          disabled={isLoading}
          bind:value={password}
        />
      </FormDialog>
    {/if}
  </Table>
</div>
